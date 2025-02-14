use crate::{
    error::{Error, Result},
    registry::{DownloadSource, SourceType},
};
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
    fn detect_file_type_from_bytes(&self, bytes: &[u8]) -> SourceType {
        let mime_type = tree_magic_mini::from_u8(bytes);
        match mime_type {
            "application/x-iso-9660-image" | "application/x-cd-image" => SourceType::Iso,

            mime if mime.contains("x-virtualbox")
                || mime.contains("x-vmdk")
                || mime.contains("x-vhd")
                || mime.contains("x-qemu") =>
            {
                SourceType::VmImage
            }

            mime if mime.starts_with("application/x-tar")
                || mime.starts_with("application/zip")
                || mime.starts_with("application/x-7z")
                || mime.starts_with("application/x-rar")
                || mime.starts_with("application/gzip")
                || mime.starts_with("application/x-bzip2")
                || mime.starts_with("application/x-xz") =>
            {
                SourceType::Archive
            }

            mime => {
                tracing::debug!("Detected unknown MIME type: {}", mime);
                SourceType::Iso
            }
        }
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
            if let Some(ref notes) = src.release_notes {
                if let Some(filename) = PathBuf::from(notes).file_name().and_then(|f| f.to_str()) {
                    return Ok(filename.to_string());
                }
            }
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

    async fn determine_path(
        &self,
        url: &str,
        first_chunk: &[u8],
        source: Option<&DownloadSource>,
        path: &PathBuf,
        output: Option<PathBuf>,
    ) -> Result<(PathBuf, SourceType)> {
        if let Some(explicit_path) = output {
            return Ok((explicit_path, self.detect_file_type_from_bytes(first_chunk)));
        }

        let file_type = self.detect_file_type_from_bytes(first_chunk);
        match source {
            Some(src) => {
                let source_dir = path
                    .join(src.source_type.to_string().to_lowercase())
                    .join(&src.name)
                    .join(&src.version);

                tokio::fs::create_dir_all(&source_dir).await?;
                let filename = self.get_download_filename(url, Some(src)).await?;
                Ok((source_dir.join(filename), file_type))
            }
            None => {
                let filename = self.get_download_filename(url, None).await?;
                let type_dir = path
                    .join("direct")
                    .join(file_type.to_string().to_lowercase());

                tokio::fs::create_dir_all(&type_dir).await?;
                Ok((type_dir.join(filename), file_type))
            }
        }
    }

    pub async fn download(
        &self,
        url: &str,
        source: Option<&DownloadSource>,
        path: &PathBuf,
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

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        let first_chunk = if let Some(chunk) = stream.next().await {
            chunk?
        } else {
            return Err(Error::EmptyContent);
        };

        let (final_path, file_type) = self
            .determine_path(url, &first_chunk, source, path, output)
            .await?;

        if final_path.exists() {
            println!("File already exists at: {}", final_path.display());
            return Ok(final_path);
        }

        if let Some(parent) = final_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let progress_bar = if self.show_progress {
            let pb = ProgressBar::new(total_size.unwrap_or(0));
            pb.enable_steady_tick(std::time::Duration::from_millis(120));
            pb.set_style(ProgressStyle::with_template(
                self.progress_style.as_deref().unwrap_or(
                    "{spinner:.green} {msg}\n[{elapsed_precise}] [{bar:40.gradient(red,yellow,green)}] {bytes:>8}/{total_bytes:8} • {binary_bytes_per_sec:>11} • ETA {eta:3}"
                )
            ).unwrap());

            if let Some(src) = source {
                pb.set_message(format!(
                    "Downloading {} {} ({})",
                    src.name,
                    src.version,
                    file_type.to_string()
                ));
            } else {
                pb.set_message(format!("Downloading {} file", file_type.to_string()));
            }
            Some(pb)
        } else {
            None
        };

        let mut file = File::create(&final_path).await?;

        file.write_all(&first_chunk).await?;
        if let Some(bar) = &progress_bar {
            downloaded += first_chunk.len() as u64;
            bar.set_position(downloaded);
        }

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            if let Some(bar) = &progress_bar {
                downloaded += chunk.len() as u64;
                bar.set_position(downloaded);
            }
        }

        file.flush().await?;

        if let Some(bar) = progress_bar {
            bar.finish_with_message(format!("Download complete: {}", final_path.display()));
        }

        Ok(final_path)
    }
}
