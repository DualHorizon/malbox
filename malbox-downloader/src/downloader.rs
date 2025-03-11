use crate::error::{Error, Result};
use crate::registry::{SourceRegistry, SourceType, SourceVariant};
use bon::Builder;
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use magic::{cookie::DatabasePaths, cookie::Flags as CookieFlags, Cookie};
use malbox_hashing::get_sha256;
use reqwest::Client;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use tokio::{fs, fs::File, io::AsyncWriteExt};
use tokio_stream::StreamExt;

#[derive(Builder)]
pub struct Downloader {
    #[builder(default = Client::new())]
    client: Client,
    #[builder(default = false)]
    show_progress: bool,
    progress_style: Option<String>,
    chunk_size: Option<usize>,
    #[builder(default = true)]
    verify_hashes: bool,
    #[builder(default = false)]
    auto_update_metadata: bool,
}

#[derive(Debug)]
pub struct DownloadResult {
    pub path: PathBuf,
    pub size: u64,
    pub sha256: String,
    pub matches_expected: Option<bool>,
}

impl Downloader {
    fn detect_file_type_from_bytes(&self, bytes: &[u8]) -> Result<SourceType> {
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
        source: Option<&SourceVariant>,
    ) -> Result<String> {
        if let Some(src) = source {
            return Ok(format!(
                "{}.{}",
                src.id,
                self.get_file_extension(&src.source_type)
            ));
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

    fn get_file_extension(&self, source_type: &SourceType) -> String {
        match source_type {
            SourceType::Iso => "iso",
            SourceType::VmImage => "img",
            SourceType::ContainerImage => "tar",
            SourceType::Archive => "zip",
        }
        .to_string()
    }

    pub async fn download(
        &self,
        url: &str,
        source: Option<&SourceVariant>,
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

        let file_type = if let Some(src) = source {
            src.source_type.clone()
        } else {
            self.detect_file_type_from_bytes(&content)?
        };

        tracing::debug!("File type detected as: {}", file_type);

        let final_path = if let Some(explicit_path) = output {
            explicit_path
        } else {
            match source {
                Some(src) => {
                    let source_dir = download_dir
                        .join(file_type.to_string().to_lowercase())
                        .join(&src.id);

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
                bar.set_message(format!("Writing {} ({})", src.id, file_type.to_string()));
            } else {
                bar.set_message(format!("Writing {} file", file_type.to_string()));
            }
        }

        if let Some(parent) = final_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let file_size = content.len() as u64;
        let mut download_result = self.compute_hashes(&content, file_size)?;
        download_result.path = final_path.clone();

        if let Some(src) = source {
            self.validate_download(&download_result, src).await?;
        }

        let mut file = File::create(&final_path).await?;
        file.write_all(&content).await?;
        file.flush().await?;

        if let Some(bar) = progress_bar {
            bar.finish_with_message(format!("Download complete: {}", final_path.display()));
        }

        if let Some(src) = source {
            self.update_registry(download_dir, src, &download_result, &final_path)
                .await?;
        }

        Ok(final_path)
    }

    fn compute_hashes(&self, content: &[u8], size: u64) -> Result<DownloadResult> {
        let mut content_clone = content.to_vec();
        let sha256_hash = get_sha256(&mut content_clone);

        Ok(DownloadResult {
            path: PathBuf::new(),
            size,
            sha256: sha256_hash,
            matches_expected: None,
        })
    }

    async fn validate_download(
        &self,
        download_result: &DownloadResult,
        source: &SourceVariant,
    ) -> Result<()> {
        if !self.verify_hashes {
            return Ok(());
        }

        if let Some(expected_hash) = &source.checksum {
            if source.checksum_type.as_deref().unwrap_or("sha256") != "sha256" {
                tracing::warn!(
                    "Only SHA256 hashes are supported, but source uses {:?}",
                    source.checksum_type
                );
            }

            let actual_hash = &download_result.sha256;

            if actual_hash != expected_hash && !self.auto_update_metadata {
                let theme = ColorfulTheme::default();

                let confirm = Confirm::with_theme(&theme)
                    .with_prompt(format!(
                        "SHA256 hash mismatch detected for {}!\nExpected: {}\nActual: {}\nContinue anyway?",
                        source.id, expected_hash, actual_hash
                    ))
                    .default(false)
                    .interact()?;

                if !confirm {
                    return Err(Error::HashMismatch(format!(
                        "SHA256 hash mismatch for {}",
                        source.id
                    )));
                }
            }
        }

        if let Some(expected_size) = source.size {
            if expected_size != download_result.size && !self.auto_update_metadata {
                let theme = ColorfulTheme::default();

                let confirm = Confirm::with_theme(&theme)
                    .with_prompt(format!(
                        "Size mismatch detected for {}!\nExpected size: {} bytes\nActual size: {} bytes\nContinue anyway?",
                        source.id, expected_size, download_result.size
                    ))
                    .default(false)
                    .interact()?;

                if !confirm {
                    return Err(Error::SizeMismatch(format!(
                        "Size mismatch for {}",
                        source.id
                    )));
                }
            }
        }

        Ok(())
    }

    async fn update_registry(
        &self,
        download_dir: &Path,
        source: &SourceVariant,
        download_result: &DownloadResult,
        file_path: &Path,
    ) -> Result<()> {
        let registry_path = download_dir.join("source_registry.json");
        let registry = SourceRegistry::load(registry_path.clone()).await?;

        let path_str = file_path.to_string_lossy().to_string();
        let now = OffsetDateTime::now_utc();

        let mut source_family = None;
        let mut source_edition = None;
        let mut source_version = None;
        let mut source_found = false;

        for family in registry.list_families() {
            for edition in &family.editions {
                for release in &edition.releases {
                    for variant in &release.variants {
                        if variant.id == source.id && variant.url == source.url {
                            source_family = Some(family.id.clone());
                            source_edition = Some(edition.id.clone());
                            source_version = Some(release.version.clone());
                            source_found = true;
                            break;
                        }
                    }
                    if source_found {
                        break;
                    }
                }
                if source_found {
                    break;
                }
            }
            if source_found {
                break;
            }
        }

        let mut updated_variant = source.clone();
        updated_variant.metadata.last_downloaded = Some(now);
        updated_variant.metadata.downloads_count += 1;
        updated_variant.metadata.local_path = Some(path_str);

        if updated_variant.size.is_none() || updated_variant.size != Some(download_result.size) {
            updated_variant.size = Some(download_result.size);
        }

        if updated_variant.checksum.is_none()
            || updated_variant.checksum.as_deref() != Some(&download_result.sha256)
        {
            updated_variant.checksum = Some(download_result.sha256.clone());
            updated_variant.checksum_type = Some("sha256".to_string());
        }

        let mut registry = SourceRegistry::load(registry_path.clone()).await?;

        if let (Some(family_id), Some(edition_id), Some(version)) =
            (source_family, source_edition, source_version)
        {
            registry.add_source(&family_id, &edition_id, &version, updated_variant)?;
        } else {
            let family_id = "custom";
            let edition_id = "download";
            let version = source.id.clone();

            registry.add_source(family_id, edition_id, &version, updated_variant)?;
        }

        registry.save(registry_path).await?;

        Ok(())
    }

    pub async fn get_source(
        &self,
        family_id: Option<&str>,
        edition_id: Option<&str>,
        version: Option<&str>,
        variant_id: Option<&str>,
        download_dir: &PathBuf,
    ) -> Result<SourceVariant> {
        let registry_path = download_dir.join("source_registry.json");
        let registry = SourceRegistry::load(registry_path).await?;

        registry.get_source(family_id, edition_id, version, variant_id)
    }

    // NOTE: Created for loose matching on sources. Should we keep something like this?
    pub async fn find_source_by_name(
        &self,
        name: &str,
        version: Option<&str>,
        download_dir: &PathBuf,
    ) -> Result<SourceVariant> {
        let registry_path = download_dir.join("source_registry.json");
        let registry = SourceRegistry::load(registry_path).await?;

        if let Ok(source) = registry.get_source(None, None, None, Some(name)) {
            return Ok(source);
        }

        if let Ok(source) = registry.get_source(Some(name), None, version, None) {
            return Ok(source);
        }

        if let Ok(source) = registry.get_source(None, Some(name), version, None) {
            return Ok(source);
        }

        if version.is_none() {
            if let Ok(source) = registry.get_source(None, None, Some(name), None) {
                return Ok(source);
            }
        }

        Err(Error::SourceNotFound(format!(
            "No source found matching name: {}",
            name
        )))
    }

    pub async fn get_source_path(
        &self,
        family_id: Option<&str>,
        edition_id: Option<&str>,
        version: Option<&str>,
        variant_id: Option<&str>,
        download_dir: &PathBuf,
    ) -> Result<Option<PathBuf>> {
        match self
            .get_source(family_id, edition_id, version, variant_id, download_dir)
            .await
        {
            Ok(source) => {
                if let Some(path_str) = &source.metadata.local_path {
                    return Ok(Some(PathBuf::from(path_str)));
                }
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }
}
