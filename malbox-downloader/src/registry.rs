use crate::error::{Error, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use time::OffsetDateTime;
use tokio::fs;

// NOTE:
// A few things here:
// 1. We need to (potentially) find a nice workaround about the fact that ValueEnum can't handle Enums that contain values
// such as Other(String)
// 2. It's IMHO - kind of ugly to add the clap ValueEnum trait here.
// We add some dependencies to the tree which could maybe be avoided.
// Open to suggestions

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum, PartialEq)]
pub enum SourceType {
    #[serde(rename = "iso")]
    Iso,
    #[serde(rename = "vm_image")]
    VmImage,
    #[serde(rename = "container_image")]
    ContainerImage,
    #[serde(rename = "archive")]
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum ProcessingStatus {
    Raw,
    PackerProcessed,
    AnsibleProvisioned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetadata {
    pub added_date: OffsetDateTime,
    pub last_verified: Option<OffsetDateTime>,
    pub last_downloaded: Option<OffsetDateTime>,
    pub downloads_count: u64,
    pub verified: bool,
    pub processing_status: ProcessingStatus,
    pub parent_source: Option<String>, // ID of source this was derived from
    pub build_info: Option<BuildInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub build_date: OffsetDateTime,
    pub build_id: String,
    pub builder_version: String,
    pub provisioner_version: Option<String>,
    pub build_parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSource {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub source_type: SourceType,
    pub metadata: SourceMetadata,
    pub checksum: Option<String>,
    pub checksum_type: Option<String>,
    pub size: Option<u64>,
    pub compression: Option<String>,
    pub platform: Platform,
    pub architecture: Architecture,
    pub minimum_requirements: Option<SystemRequirements>,
    pub tags: Vec<String>,
    pub mirrors: Vec<String>,
    pub license: Option<String>,
    pub documentation_url: Option<String>,
    pub release_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub disk_gb: u64,
    pub additional_requirements: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum Platform {
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "macos")]
    MacOS,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum Architecture {
    #[serde(rename = "x86")]
    X86,
    #[serde(rename = "x86_64")]
    X86_64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadRegistry {
    pub sources: HashMap<String, Vec<DownloadSource>>,
    pub custom_sources: HashMap<String, DownloadSource>,
}

impl DownloadRegistry {
    pub async fn load(registry_path: PathBuf) -> Result<DownloadRegistry> {
        if !registry_path.exists() {
            let default_registry = Self {
                sources: Self::default_sources(),
                custom_sources: HashMap::new(),
            };

            let content = serde_json::to_string_pretty(&default_registry)
                .map_err(|e| Error::InvalidData(e.to_string()))?;

            fs::write(&registry_path, content).await?;
            return Ok(default_registry);
        }

        let content = fs::read_to_string(registry_path).await?;
        serde_json::from_str(&content).map_err(|e| Error::InvalidData(e.to_string()))
    }

    pub async fn save(&self, registry_path: PathBuf) -> Result<()> {
        let content =
            serde_json::to_string_pretty(self).map_err(|e| Error::InvalidData(e.to_string()))?;
        fs::write(registry_path, content).await?;

        Ok(())
    }

    pub fn get_source(&self, name: &str, version: Option<&str>) -> Result<DownloadSource> {
        // First check custom sources
        if let Some(source) = self.custom_sources.get(name) {
            if let Some(ver) = version {
                if source.version == ver {
                    return Ok(source.clone());
                }
            } else {
                return Ok(source.clone());
            }
        }

        // Then check predefined sources
        for (category, sources) in &self.sources {
            for source in sources {
                if source.name == name {
                    if let Some(ver) = version {
                        if source.version == ver {
                            return Ok(source.clone());
                        }
                    } else {
                        return Ok(source.clone());
                    }
                }
            }
        }

        Err(Error::SourceNotFound(name.to_string()))
    }

    pub fn add_custom_source(&mut self, source: DownloadSource) {
        let key = format!("{}-{}", source.name, source.version);
        self.custom_sources.insert(key, source);
    }

    fn default_sources() -> HashMap<String, Vec<DownloadSource>> {
        let mut sources = HashMap::new();
        let now = OffsetDateTime::now_utc();

        sources.insert(
            "windows".to_string(),
            vec![
                DownloadSource {
                    name: "windows".to_string(),
                    version: "10-22h2-64".to_string(),
                    description: "Windows 10 22H2 x64".to_string(),
                    url: "https://example.com/win10-22h2-64.iso".to_string(),
                    source_type: SourceType::Iso,
                    metadata: SourceMetadata {
                        added_date: now,
                        last_verified: Some(now),
                        last_downloaded: None,
                        downloads_count: 0,
                        verified: true,
                        processing_status: ProcessingStatus::Raw,
                        parent_source: None,
                        build_info: None,
                    },
                    checksum: Some("abc123".to_string()),
                    checksum_type: Some("sha256".to_string()),
                    size: Some(5_368_709_120),
                    compression: None,
                    platform: Platform::Windows,
                    architecture: Architecture::X86_64,
                    minimum_requirements: Some(SystemRequirements {
                        cpu_cores: 2,
                        memory_mb: 4096,
                        disk_gb: 64,
                        additional_requirements: None,
                    }),
                    tags: vec!["windows".to_string(), "desktop".to_string()],
                    mirrors: vec![],
                    license: Some("Microsoft Windows License".to_string()),
                    documentation_url: Some("https://docs.microsoft.com/windows".to_string()),
                    release_notes: None,
                },
                // Add more Windows sources...
            ],
        );

        // Linux sources
        sources.insert(
            "linux".to_string(),
            vec![DownloadSource {
                name: "ubuntu".to_string(),
                version: "22.04-server-64".to_string(),
                description: "Ubuntu 22.04 LTS Server x64".to_string(),
                url: "https://releases.ubuntu.com/22.04/ubuntu-22.04-live-server-amd64.iso"
                    .to_string(),
                source_type: SourceType::Iso,
                metadata: SourceMetadata {
                    added_date: now,
                    last_verified: Some(now),
                    last_downloaded: None,
                    downloads_count: 0,
                    verified: true,
                    processing_status: ProcessingStatus::Raw,
                    parent_source: None,
                    build_info: None,
                },
                checksum: None,
                checksum_type: None,
                size: None,
                compression: None,
                platform: Platform::Linux,
                architecture: Architecture::X86_64,
                minimum_requirements: Some(SystemRequirements {
                    cpu_cores: 1,
                    memory_mb: 2048,
                    disk_gb: 25,
                    additional_requirements: None,
                }),
                tags: vec!["linux".to_string(), "server".to_string()],
                mirrors: vec![],
                license: Some("GPL".to_string()),
                documentation_url: Some("https://ubuntu.com/server/docs".to_string()),
                release_notes: Some("https://ubuntu.com/server/releases/22.04".to_string()),
            }],
        );

        sources
    }
}
