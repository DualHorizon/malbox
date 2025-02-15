use crate::error::{Error, Result};
use crate::registry::{DownloadSource, SourceType};
use bon::Builder;
use indicatif::{ProgressBar, ProgressStyle};
use magic::{cookie::DatabasePaths, cookie::Flags as CookieFlags, Cookie};
use reqwest::Client;
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_stream::StreamExt;

#[derive(Builder)]
pub struct Downloader {
    #[builder(default = Client::new())]
    client: Client,
    #[builder(default = false)]
    show_progress: bool,
    progress_style: Option<String>,
    chunk_size: Option<usize>,
}

impl Downloader {
    fn detect_file_type_from_bytes(&self, bytes: &[u8]) -> Result<SourceType> {
        // NOTE:
        // It is probably better if we find another crate better suited for this, I don't really
        // like the way we match file types here. Note that the `tree_magic_mini` crate didn't properly
        // recognize file types.

        let cookie = Cookie::open(CookieFlags::default())
            .map_err(|e| Error::Detection(format!("Failed to open magic cookie: {}", e)))?;

        let cookie = cookie
            .load(&DatabasePaths::default())
            .map_err(|e| Error::Detection(format!("Failed to load magic database: {}", e)))?;

        let file_type = cookie
            .buffer(bytes)
            .map_err(|e| Error::Detection(format!("Failed to analyze file type: {}", e)))?;

        tracing::debug!("Magic detected file type: {}", file_type);

        Ok(match file_type.as_str() {
            type_str if type_str.contains("ISO 9660") => SourceType::Iso,
            type_str
                if type_str.contains("VMware")
                    || type_str.contains("VirtualBox")
                    || type_str.contains("QEMU") =>
            {
                SourceType::VmImage
            }
            type_str
                if type_str.contains("gzip")
                    || type_str.contains("bzip2")
                    || type_str.contains("Zip")
                    || type_str.contains("RAR") =>
            {
                SourceType::Archive
            }
            type_str => {
                return Err(Error::Detection(format!(
                    "File type: {} is not supported",
                    type_str
                )));
            }
        })
    }

    async fn get_filename_from_headers(&self, response: &reqwest::Response) -> Option<String> {
        response
            .headers()
            .get("content-disposition")
            .and_then(|cd| cd.to_str().ok())
            .and_then(|cd| {
                cd.split("filename=")
                    .nth(1)
                    .map(|f| f.trim_matches('"').to_string())
            })
    }

    async fn get_download_filename(
        &self,
        url: &str,
        source: Option<&DownloadSource>,
    ) -> Result<String> {
        if let Some(src) = source {
            return Ok(format!("{}-{}.iso", src.name, src.version));
        }

        let response = self.client.get(url).send().await?;

        if let Some(filename) = self.get_filename_from_headers(&response).await {
            return Ok(filename);
        }

        if let Some(filename) = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .map(|s| s.to_string())
        {
            return Ok(filename);
        }

        Ok("download.bin".to_string())
    }
    pub async fn download(
        &self,
        url: &str,
        source: Option<&DownloadSource>,
        download_dir: &PathBuf,
        output: Option<PathBuf>,
    ) -> Result<PathBuf> {
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(Error::HttpStatus(response.status()));
        }

        let total_size = response.content_length();
        if total_size == Some(0) {
            return Err(Error::EmptyContent);
        }

        let progress_bar = if self.show_progress {
            let pb = ProgressBar::new(total_size.unwrap_or(0));
            pb.enable_steady_tick(std::time::Duration::from_millis(120));
            pb.set_style(ProgressStyle::with_template(
                self.progress_style.as_deref().unwrap_or(
                    "{spinner:.green} {msg}\n[{elapsed_precise}] [{bar:40.gradient(red,yellow,green)}] {bytes:>8}/{total_bytes:8} • {binary_bytes_per_sec:>11} • ETA {eta:3}"
                )
            ).unwrap());
            pb.set_message("Downloading file...");
            Some(pb)
        } else {
            None
        };

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut content = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            content.extend_from_slice(&chunk);
            if let Some(bar) = &progress_bar {
                downloaded += chunk.len() as u64;
                bar.set_position(downloaded);
            }
        }

        let file_type = self.detect_file_type_from_bytes(&content)?;
        // FIXME:
        // For some reason, this trace won't print out!
        // Will def. need some investigation.
        tracing::debug!("File type detected as: {}", file_type);

        let final_path = if let Some(explicit_path) = output {
            explicit_path
        } else {
            match source {
                Some(src) => {
                    let source_dir = download_dir
                        .join(src.source_type.to_string().to_lowercase())
                        .join(&src.name)
                        .join(&src.version);

                    tokio::fs::create_dir_all(&source_dir).await?;
                    let filename = self.get_download_filename(url, Some(src)).await?;
                    source_dir.join(filename)
                }
                None => {
                    let filename = self.get_download_filename(url, None).await?;
                    let type_dir = download_dir
                        .join("direct")
                        .join(file_type.to_string().to_lowercase());

                    tokio::fs::create_dir_all(&type_dir).await?;
                    type_dir.join(filename)
                }
            }
        };

        if final_path.exists() {
            println!("File already exists at: {}", final_path.display());
            return Ok(final_path);
        }

        if let Some(bar) = &progress_bar {
            if let Some(src) = source {
                bar.set_message(format!(
                    "Writing {} {} ({})",
                    src.name,
                    src.version,
                    file_type.to_string()
                ));
            } else {
                bar.set_message(format!("Writing {} file", file_type.to_string()));
            }
        }

        if let Some(parent) = final_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = File::create(&final_path).await?;
        file.write_all(&content).await?;
        file.flush().await?;

        if let Some(bar) = progress_bar {
            bar.finish_with_message(format!("Download complete: {}", final_path.display()));
        }

        Ok(final_path)
    }
}
