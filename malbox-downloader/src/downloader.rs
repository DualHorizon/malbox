use crate::error::{Error, Result};
use bon::Builder;
use indicatif::{ProgressBar, ProgressStyle};
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
    pub async fn download(&self, url: &str, path: PathBuf) -> Result<()> {
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
                    "{spinner:.green} {bar:40.gradient(red,yellow,green)} {bytes:>8}/{total_bytes:8} • {binary_bytes_per_sec:>11} • ETA {eta:3}"
                )
            ).unwrap());
            Some(pb)
        } else {
            None
        };

        let mut file = File::create(path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;

            if let Some(bar) = &progress_bar {
                downloaded += chunk.len() as u64;
                bar.set_position(downloaded);
            }
        }

        if let Some(bar) = progress_bar {
            bar.finish_with_message("Download complete");
        }

        file.flush().await?;
        Ok(())
    }
}
