use crate::error::{IoError, Result};
use futures::{Future, FutureExt, Stream, StreamExt};
use std::collections::HashMap;
use std::ffi::{CString, OsStr};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio_stream::wrappers::LinesStream;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum OutputSource {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct OutputLine {
    pub source: OutputSource,
    pub content: String,
}

pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub combined_output: Vec<OutputLine>,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    pub fn stdout(&self) -> String {
        self.stdout_lines.join("\n")
    }

    pub fn stderr(&self) -> String {
        self.stderr_lines.join("\n")
    }

    pub fn combined(&self) -> String {
        self.combined_output
            .iter()
            .map(|line| line.content.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub struct AsyncCommand {
    inner: tokio::process::Command,
    stdin: bool,
    stdout: bool,
    stderr: bool,
}

impl AsyncCommand {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self {
            inner: tokio::process::Command::new(program),
            stdin: false,
            stdout: true,
            stderr: true,
        }
    }

    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.inner.arg(arg);
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }

    pub fn current_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.inner.current_dir(dir);
        self
    }

    pub fn env<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.inner.env(key, value);
        self
    }

    pub async fn output_stream(
        &mut self,
    ) -> Result<(
        Pin<Box<dyn Stream<Item = OutputLine> + Send>>,
        Pin<Box<dyn Future<Output = i32> + Send>>,
    )> {
        self.inner.stdout(Stdio::piped());
        self.inner.stderr(Stdio::piped());

        let mut child = self.inner.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                IoError::CommandNotFound {
                    command: format!("{:?}", self.inner),
                }
            } else {
                IoError::SpawnFailed {
                    command: format!("{:?}", self.inner),
                    message: e.to_string(),
                }
            }
        })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| IoError::OutputProcessing {
                message: "Failed to capture stdout".to_string(),
            })?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| IoError::OutputProcessing {
                message: "Failed to capture stderr".to_string(),
            })?;

        let stdout_lines =
            LinesStream::new(BufReader::new(stdout).lines()).map(|line_result| match line_result {
                Ok(line) => OutputLine {
                    source: OutputSource::Stdout,
                    content: line,
                },
                Err(e) => {
                    error!("Error reading stdout: {}", e);
                    OutputLine {
                        source: OutputSource::Stdout,
                        content: format!("[Error reading output: {}]", e),
                    }
                }
            });

        let stderr_lines =
            LinesStream::new(BufReader::new(stderr).lines()).map(|line_result| match line_result {
                Ok(line) => OutputLine {
                    source: OutputSource::Stderr,
                    content: line,
                },
                Err(e) => {
                    error!("Error reading stderr: {}", e);
                    OutputLine {
                        source: OutputSource::Stderr,
                        content: format!("[Error reading output: {}]", e),
                    }
                }
            });

        let output_stream = futures::stream::select(stdout_lines, stderr_lines).boxed();

        let exit_code_future = async move {
            match child.wait().await {
                Ok(status) => status.code().unwrap_or(-1),
                Err(e) => {
                    error!("Error waiting for process to exit: {}", e);
                    -1
                }
            }
        }
        .boxed();

        Ok((output_stream, exit_code_future))
    }

    pub async fn run_with_output_handler<F>(
        &mut self,
        mut output_handler: F,
    ) -> Result<CommandOutput>
    where
        F: FnMut(&OutputLine),
    {
        let (mut output_stream, exit_code_future) = self.output_stream().await?;

        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();
        let mut combined_output = Vec::new();

        while let Some(line) = output_stream.next().await {
            output_handler(&line);

            match line.source {
                OutputSource::Stdout => stdout_lines.push(line.content.clone()),
                OutputSource::Stderr => stderr_lines.push(line.content.clone()),
            }

            combined_output.push(line);
        }

        let exit_code = exit_code_future.await;

        Ok(CommandOutput {
            exit_code,
            stdout_lines,
            stderr_lines,
            combined_output,
        })
    }

    pub async fn run(&mut self) -> Result<CommandOutput> {
        self.run_with_output_handler(|_| {}).await
    }

    pub async fn run_with_standard_logging(&mut self) -> Result<CommandOutput> {
        self.run_with_output_handler(|line| {
            let content = &line.content;
            match line.source {
                OutputSource::Stderr => {
                    if content.contains("error:") || content.contains("Error:") {
                        error!("[CMD ERROR] {}", content);
                    } else {
                        warn!("[CMD STDERR] {}", content);
                    }
                }
                OutputSource::Stdout => {
                    if content.contains("warning:") || content.contains("Warning:") {
                        warn!("[CMD WARNING] {}", content);
                    } else {
                        info!("[CMD] {}", content);
                    }
                }
            }
        })
        .await
    }
}
