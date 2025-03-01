use crate::error::{Error, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use tokio::fs;

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

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::Iso => write!(f, "ISO"),
            SourceType::VmImage => write!(f, "VM Image"),
            SourceType::ContainerImage => write!(f, "Container Image"),
            SourceType::Archive => write!(f, "Archive"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum ProcessingStatus {
    Raw,
    PackerProcessed,
    AnsibleProvisioned,
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
pub struct SourceMetadata {
    pub added_date: OffsetDateTime,
    pub last_verified: Option<OffsetDateTime>,
    pub last_downloaded: Option<OffsetDateTime>,
    pub downloads_count: u64,
    pub verified: bool,
    pub processing_status: ProcessingStatus,
    pub parent_source: Option<String>,
    pub build_info: Option<BuildInfo>,
    pub local_path: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum, PartialEq)]
pub enum Architecture {
    #[serde(rename = "x86")]
    X86,
    #[serde(rename = "x86-64")]
    X86_64,
    #[serde(rename = "arm64")]
    Arm64,
}

// Hierarchical Source Model

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFamily {
    pub id: String,
    pub name: String,
    pub description: String,
    pub platform: Platform,
    pub editions: Vec<SourceEdition>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEdition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub releases: Vec<SourceRelease>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRelease {
    pub version: String,
    pub release_date: Option<OffsetDateTime>,
    pub description: String,
    pub release_notes: Option<String>,
    pub eol_date: Option<OffsetDateTime>,
    pub variants: Vec<SourceVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceVariant {
    pub id: String,
    pub description: String,
    pub architecture: Architecture,
    pub url: String,
    pub checksum: Option<String>,
    pub checksum_type: Option<String>,
    pub size: Option<u64>,
    pub source_type: SourceType,
    pub compression: Option<String>,
    pub metadata: SourceMetadata,
    pub minimum_requirements: Option<SystemRequirements>,
    pub mirrors: Vec<String>,
    pub license: Option<String>,
    pub documentation_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceRegistry {
    pub families: HashMap<String, SourceFamily>,
    pub custom_families: HashMap<String, SourceFamily>,
}

impl SourceRegistry {
    pub async fn load(registry_path: PathBuf) -> Result<SourceRegistry> {
        if !registry_path.exists() {
            let default_registry = Self {
                families: Self::default_families(),
                custom_families: HashMap::new(),
            };

            let content = serde_json::to_string_pretty(&default_registry)
                .map_err(|e| Error::InvalidData(e.to_string()))?;

            fs::write(&registry_path, content).await?;
            return Ok(default_registry);
        }

        let content = fs::read_to_string(&registry_path).await?;
        let registry = serde_json::from_str::<SourceRegistry>(&content)
            .map_err(|e| Error::InvalidData(e.to_string()))?;

        Ok(registry)
    }

    pub async fn save(&self, registry_path: PathBuf) -> Result<()> {
        let content =
            serde_json::to_string_pretty(self).map_err(|e| Error::InvalidData(e.to_string()))?;
        fs::write(registry_path, content).await?;
        Ok(())
    }

    pub fn get_source(
        &self,
        family_id: Option<&str>,
        edition_id: Option<&str>,
        version: Option<&str>,
        variant_id: Option<&str>,
    ) -> Result<SourceVariant> {
        let sources = self.find_sources(family_id, edition_id, version, variant_id)?;

        if sources.is_empty() {
            return Err(Error::SourceNotFound(format!(
            "No source found matching criteria: family={:?}, edition={:?}, version={:?}, variant={:?}",
            family_id, edition_id, version, variant_id
        )));
        }

        // Sort by recency and return the most recent one
        let mut variants = sources;
        variants.sort_by(|a, b| {
            let a_date = a.metadata.last_verified.unwrap_or(a.metadata.added_date);
            let b_date = b.metadata.last_verified.unwrap_or(b.metadata.added_date);
            b_date.cmp(&a_date) // Newest first
        });

        Ok(variants[0].clone())
    }

    fn find_sources(
        &self,
        family_id: Option<&str>,
        edition_id: Option<&str>,
        version: Option<&str>,
        variant_id: Option<&str>,
    ) -> Result<Vec<SourceVariant>> {
        let mut results = Vec::new();

        // Search in both custom and standard families
        for family_map in [&self.custom_families, &self.families] {
            for (fam_id, family) in family_map {
                // Skip if family_id specified and doesn't match
                if let Some(f_id) = family_id {
                    if fam_id != f_id {
                        continue;
                    }
                }

                for edition in &family.editions {
                    // Skip if edition_id specified and doesn't match
                    if let Some(e_id) = edition_id {
                        if edition.id != e_id {
                            continue;
                        }
                    }

                    for release in &edition.releases {
                        // Skip if version specified and doesn't match
                        if let Some(v) = version {
                            if release.version != v {
                                continue;
                            }
                        }

                        for variant in &release.variants {
                            // Skip if variant_id specified and doesn't match
                            if let Some(v_id) = variant_id {
                                if variant.id != v_id {
                                    continue;
                                }
                            }

                            // Simply add the matching variant to results
                            results.push(variant.clone());
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            return Err(Error::SourceNotFound(format!(
            "No sources matching criteria: family={:?}, edition={:?}, version={:?}, variant={:?}",
            family_id, edition_id, version, variant_id
        )));
        }

        Ok(results)
    }

    pub fn add_source(
        &mut self,
        family_id: &str,
        edition_id: &str,
        version: &str,
        variant: SourceVariant,
    ) -> Result<()> {
        let family = self
            .custom_families
            .entry(family_id.to_string())
            .or_insert_with(|| {
                let platform = match family_id {
                    "windows" => Platform::Windows,
                    "linux" => Platform::Linux,
                    "macos" => Platform::MacOS,
                    _ => Platform::Linux,
                };

                SourceFamily {
                    id: family_id.to_string(),
                    name: family_id.to_string(),
                    description: format!("{} sources", family_id),
                    platform,
                    editions: Vec::new(),
                    tags: vec![family_id.to_string()],
                }
            });

        let edition = match family.editions.iter_mut().find(|e| e.id == edition_id) {
            Some(edition) => edition,
            None => {
                family.editions.push(SourceEdition {
                    id: edition_id.to_string(),
                    name: edition_id.to_string(),
                    description: format!("{} edition", edition_id),
                    releases: Vec::new(),
                });
                family.editions.last_mut().unwrap()
            }
        };

        let release = match edition.releases.iter_mut().find(|r| r.version == version) {
            Some(release) => release,
            None => {
                edition.releases.push(SourceRelease {
                    version: version.to_string(),
                    release_date: None,
                    description: format!("Version {}", version),
                    release_notes: None,
                    eol_date: None,
                    variants: Vec::new(),
                });
                edition.releases.last_mut().unwrap()
            }
        };

        if let Some(existing) = release.variants.iter_mut().find(|v| v.id == variant.id) {
            *existing = variant;
        } else {
            release.variants.push(variant);
        }

        Ok(())
    }

    pub fn get_filename_for_source_type(&self, source_type: &SourceType) -> String {
        match source_type {
            SourceType::Iso => "image.iso",
            SourceType::VmImage => "image.img",
            SourceType::ContainerImage => "image.tar",
            SourceType::Archive => "archive.zip",
        }
        .to_string()
    }

    pub fn source_exists(
        &self,
        family_id: Option<&str>,
        edition_id: Option<&str>,
        version: Option<&str>,
        variant_id: Option<&str>,
    ) -> bool {
        self.get_source(family_id, edition_id, version, variant_id)
            .is_ok()
    }

    pub fn list_families(&self) -> Vec<&SourceFamily> {
        let mut families = Vec::new();
        families.extend(self.families.values());
        families.extend(self.custom_families.values());
        families.sort_by(|a, b| a.id.cmp(&b.id));
        families
    }

    pub fn list_editions(&self, family_id: &str) -> Result<Vec<&SourceEdition>> {
        let mut editions = Vec::new();

        if let Some(family) = self.custom_families.get(family_id) {
            editions.extend(family.editions.iter());
        }

        if let Some(family) = self.families.get(family_id) {
            editions.extend(family.editions.iter());
        }

        if editions.is_empty() {
            return Err(Error::SourceNotFound(family_id.to_string()));
        }

        editions.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(editions)
    }

    pub fn list_releases(&self, family_id: &str, edition_id: &str) -> Result<Vec<&SourceRelease>> {
        let mut releases = Vec::new();

        if let Some(family) = self.custom_families.get(family_id) {
            for edition in &family.editions {
                if edition.id == edition_id {
                    releases.extend(edition.releases.iter());
                }
            }
        }

        if let Some(family) = self.families.get(family_id) {
            for edition in &family.editions {
                if edition.id == edition_id {
                    releases.extend(edition.releases.iter());
                }
            }
        }

        if releases.is_empty() {
            return Err(Error::SourceNotFound(format!(
                "{}/{}",
                family_id, edition_id
            )));
        }

        releases.sort_by(|a, b| a.version.cmp(&b.version));
        Ok(releases)
    }

    pub fn list_variants(
        &self,
        family_id: &str,
        edition_id: &str,
        version: &str,
    ) -> Result<Vec<&SourceVariant>> {
        let mut variants = Vec::new();

        if let Some(family) = self.custom_families.get(family_id) {
            for edition in &family.editions {
                if edition.id == edition_id {
                    for release in &edition.releases {
                        if release.version == version {
                            variants.extend(release.variants.iter());
                        }
                    }
                }
            }
        }

        if let Some(family) = self.families.get(family_id) {
            for edition in &family.editions {
                if edition.id == edition_id {
                    for release in &edition.releases {
                        if release.version == version {
                            variants.extend(release.variants.iter());
                        }
                    }
                }
            }
        }

        if variants.is_empty() {
            return Err(Error::SourceNotFound(format!(
                "{}/{}/{}",
                family_id, edition_id, version
            )));
        }

        variants.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(variants)
    }

    pub fn search(&self, query: &str) -> Vec<SourceVariant> {
        let query = query.to_lowercase();
        let mut results = Vec::new();

        for family in self.list_families() {
            for edition in &family.editions {
                for release in &edition.releases {
                    for variant in &release.variants {
                        if variant.description.to_lowercase().contains(&query)
                            || family.id.to_lowercase().contains(&query)
                            || edition.id.to_lowercase().contains(&query)
                            || release.version.to_lowercase().contains(&query)
                            || variant.id.to_lowercase().contains(&query)
                        {
                            results.push(variant.clone());
                        }
                    }
                }
            }
        }

        results
    }

    pub fn delete_source(
        &mut self,
        family_id: &str,
        edition_id: &str,
        version: &str,
        variant_id: &str,
    ) -> Result<()> {
        if let Some(family) = self.custom_families.get_mut(family_id) {
            for edition_idx in 0..family.editions.len() {
                if family.editions[edition_idx].id == edition_id {
                    for release_idx in 0..family.editions[edition_idx].releases.len() {
                        if family.editions[edition_idx].releases[release_idx].version == version {
                            let release = &mut family.editions[edition_idx].releases[release_idx];

                            release.variants.retain(|v| v.id != variant_id);

                            if release.variants.is_empty() {
                                family.editions[edition_idx].releases.remove(release_idx);

                                if family.editions[edition_idx].releases.is_empty() {
                                    family.editions.remove(edition_idx);

                                    if family.editions.is_empty() {
                                        self.custom_families.remove(family_id);
                                    }
                                }
                            }

                            return Ok(());
                        }
                    }
                }
            }
        }

        Err(Error::SourceNotFound(format!(
            "{}/{}/{}/{}",
            family_id, edition_id, version, variant_id
        )))
    }

    pub fn purge_custom_sources(&mut self) {
        self.custom_families.clear();
    }

    pub fn get_all_sources(&self) -> Vec<SourceVariant> {
        let mut sources = Vec::new();

        for family in self.list_families() {
            for edition in &family.editions {
                for release in &edition.releases {
                    for variant in &release.variants {
                        sources.push(variant.clone());
                    }
                }
            }
        }

        sources
    }

    pub fn get_sources_with_local_paths(&self) -> Vec<SourceVariant> {
        self.get_all_sources()
            .into_iter()
            .filter(|variant| variant.metadata.local_path.is_some())
            .collect()
    }

    fn default_families() -> HashMap<String, SourceFamily> {
        let mut families = HashMap::new();
        let now = OffsetDateTime::now_utc();

        let windows = SourceFamily {
            id: "windows".to_string(),
            name: "Microsoft Windows".to_string(),
            description: "Windows operating systems".to_string(),
            platform: Platform::Windows,
            tags: vec!["windows".to_string()],
            editions: vec![SourceEdition {
                id: "windows".to_string(),
                name: "Windows".to_string(),
                description: "Windows desktop editions".to_string(),
                releases: vec![SourceRelease {
                    version: "10-22h2".to_string(),
                    release_date: Some(now),
                    description: "Windows 10 Version 22H2".to_string(),
                    release_notes: None,
                    eol_date: None,
                    variants: vec![SourceVariant {
                        id: "x64".to_string(),
                        description: "Windows 10 22H2 x64".to_string(),
                        architecture: Architecture::X86_64,
                        url: "https://example.com/win10-22h2-64.iso".to_string(),
                        checksum: Some("abc123".to_string()),
                        checksum_type: Some("sha256".to_string()),
                        size: Some(5_368_709_120),
                        source_type: SourceType::Iso,
                        compression: None,
                        metadata: SourceMetadata {
                            added_date: now,
                            last_verified: Some(now),
                            last_downloaded: None,
                            downloads_count: 0,
                            verified: true,
                            processing_status: ProcessingStatus::Raw,
                            parent_source: None,
                            build_info: None,
                            local_path: None,
                        },
                        minimum_requirements: Some(SystemRequirements {
                            cpu_cores: 2,
                            memory_mb: 4096,
                            disk_gb: 64,
                            additional_requirements: None,
                        }),
                        mirrors: vec![],
                        license: Some("Microsoft Windows License".to_string()),
                        documentation_url: Some("https://docs.microsoft.com/windows".to_string()),
                    }],
                }],
            }],
        };

        let linux = SourceFamily {
            id: "linux".to_string(),
            name: "Linux".to_string(),
            description: "Linux operating systems".to_string(),
            platform: Platform::Linux,
            tags: vec!["linux".to_string()],
            editions: vec![SourceEdition {
                id: "ubuntu".to_string(),
                name: "Ubuntu".to_string(),
                description: "Ubuntu Linux distribution".to_string(),
                releases: vec![SourceRelease {
                    version: "22.04".to_string(),
                    release_date: Some(now),
                    description: "Ubuntu 22.04 LTS".to_string(),
                    release_notes: Some("https://ubuntu.com/server/releases/22.04".to_string()),
                    eol_date: None,
                    variants: vec![SourceVariant {
                        id: "server-x64".to_string(),
                        description: "Ubuntu 22.04 LTS Server x64".to_string(),
                        architecture: Architecture::X86_64,
                        url: "https://releases.ubuntu.com/22.04/ubuntu-22.04-live-server-amd64.iso"
                            .to_string(),
                        checksum: None,
                        checksum_type: None,
                        size: None,
                        source_type: SourceType::Iso,
                        compression: None,
                        metadata: SourceMetadata {
                            added_date: now,
                            last_verified: Some(now),
                            last_downloaded: None,
                            downloads_count: 0,
                            verified: true,
                            processing_status: ProcessingStatus::Raw,
                            parent_source: None,
                            build_info: None,
                            local_path: None,
                        },
                        minimum_requirements: Some(SystemRequirements {
                            cpu_cores: 1,
                            memory_mb: 2048,
                            disk_gb: 25,
                            additional_requirements: None,
                        }),
                        mirrors: vec![],
                        license: Some("GPL".to_string()),
                        documentation_url: Some("https://ubuntu.com/server/docs".to_string()),
                    }],
                }],
            }],
        };

        families.insert("windows".to_string(), windows);
        families.insert("linux".to_string(), linux);

        families
    }
}
