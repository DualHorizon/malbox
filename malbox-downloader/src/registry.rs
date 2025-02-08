use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadSource {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub checksum: Option<String>,
    pub checksum_type: Option<String>,
    pub size: Option<u64>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub mirrors: Vec<String>,
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

        // Windows sources
        sources.insert(
            "windows".to_string(),
            vec![
                DownloadSource {
                    name: "windows".to_string(),
                    version: "10-22h2-64".to_string(),
                    description: "Windows 10 22H2 x64".to_string(),
                    url: "https://example.com/win10-22h2-64.iso".to_string(),
                    checksum: Some("abc123".to_string()),
                    checksum_type: Some("sha256".to_string()),
                    size: Some(5_368_709_120),
                    tags: vec!["windows".to_string(), "desktop".to_string()],
                    mirrors: vec![],
                },
                DownloadSource {
                    name: "windows".to_string(),
                    version: "11-23h2-64".to_string(),
                    description: "Windows 11 23H2 x64".to_string(),
                    url: "https://example.com/win11-23h2-64.iso".to_string(),
                    checksum: Some("def456".to_string()),
                    checksum_type: Some("sha256".to_string()),
                    size: Some(6_442_450_944),
                    tags: vec!["windows".to_string(), "desktop".to_string()],
                    mirrors: vec![],
                },
            ],
        );

        // Linux sources
        sources.insert("linux".to_string(), vec![
            DownloadSource {
                name: "ubuntu".to_string(),
                version: "22.04-server-64".to_string(),
                description: "Ubuntu 22.04 LTS Server x64".to_string(),
                url: "https://releases.ubuntu.com/22.04/ubuntu-22.04-live-server-amd64.iso".to_string(),
                checksum: None,
                checksum_type: None,
                size: None,
                tags: vec!["linux".to_string(), "server".to_string()],
                mirrors: vec![],
            },
            DownloadSource {
                name: "debian".to_string(),
                version: "12-64".to_string(),
                description: "Debian 12 x64".to_string(),
                url: "https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/debian-12.0.0-amd64-netinst.iso".to_string(),
                checksum: None,
                checksum_type: None,
                size: None,
                tags: vec!["linux".to_string(), "server".to_string()],
                mirrors: vec![],
            },
        ]);

        sources
    }
}
