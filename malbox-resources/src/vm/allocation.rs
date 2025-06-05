use crate::{
    allocation::{AllocationRequest, AllocationStrategy},
    error::Result,
    types::Resource,
};
use malbox_database::repositories::machinery::MachinePlatform;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// VM-specific allocation strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmAllocationStrategy {
    /// Prefer VMs with matching platform.
    PlatformAware,
    /// Prefer VMs with lower resource utilization.
    ResourceEfficient,
    /// Prefer VMs in the same network/region.
    NetworkAware,
    /// Prefer VMs with specific snapshots.
    SnapshotAware,
    /// Custom strategy with weighted scoring.
    Weighted {
        platform_weight: f64,
        resource_weight: f64,
        network_weight: f64,
        snapshot_weight: f64,
    },
}

impl Default for VmAllocationStrategy {
    fn default() -> Self {
        VmAllocationStrategy::PlatformAware
    }
}

impl VmAllocationStrategy {
    /// Create a weighted strategy with custom weights.
    pub fn weighted(
        platform_weight: f64,
        resource_weight: f64,
        network_weight: f64,
        snapshot_weight: f64,
    ) -> Self {
        Self::Weighted {
            platform_weight,
            resource_weight,
            network_weight,
            snapshot_weight,
        }
    }

    /// Calculate platform match score.
    fn platform_score(&self, resource: &Resource, request: &AllocationRequest) -> f64 {
        if let Some(required_platform) = &request.constraints.platform {
            if let Some(resource_platform) = resource.platform() {
                if resource_platform == required_platform {
                    return 1.0;
                }
            }
            return 0.0;
        }
        0.5 // Neutral score if no platform preference
    }

    /// Calculate resource efficiency score.
    fn resource_efficiency_score(&self, resource: &Resource, request: &AllocationRequest) -> f64 {
        let mut score = 0.0;

        // Score based on CPU efficiency
        if let (Some(required_cpu), Some(available_cpu)) =
            (request.constraints.min_cpu_cores, resource.cpu_cores())
        {
            if available_cpu >= required_cpu {
                // Prefer resources that aren't overly powerful (avoid waste)
                let efficiency = required_cpu as f64 / available_cpu as f64;
                score += efficiency * 0.5;
            }
        }

        // Score based on memory efficiency
        if let (Some(required_memory), Some(available_memory)) =
            (request.constraints.min_memory_mb, resource.memory_mb())
        {
            if available_memory >= required_memory {
                let efficiency = required_memory as f64 / available_memory as f64;
                score += efficiency * 0.5;
            }
        }

        score.min(1.0)
    }

    /// Calculate network awareness score.
    fn network_score(&self, resource: &Resource, _request: &AllocationRequest) -> f64 {
        // For now, prefer resources with known IP addresses
        if resource.properties.contains_key("ip_address") {
            1.0
        } else {
            0.0
        }
    }

    /// Calculate snapshot awareness score.
    fn snapshot_score(&self, resource: &Resource, request: &AllocationRequest) -> f64 {
        let has_snapshot = resource.properties.contains_key("snapshot");

        // Check if request has snapshot preferences in custom constraints
        let prefers_snapshot = request
            .constraints
            .custom
            .get("snapshot")
            .map(|s| !s.is_empty())
            .unwrap_or(false);

        match (has_snapshot, prefers_snapshot) {
            (true, true) => 1.0,
            (false, false) => 0.8,
            _ => 0.3,
        }
    }
}

#[async_trait::async_trait]
impl AllocationStrategy for VmAllocationStrategy {
    async fn select_resource(
        &self,
        request: &AllocationRequest,
        available_resources: &[Resource],
    ) -> Result<Option<Resource>> {
        if available_resources.is_empty() {
            return Ok(None);
        }

        debug!("Selecting VM resource using strategy: {:?}", self);

        let mut scored_resources: Vec<(Resource, f64)> = available_resources
            .iter()
            .map(|resource| {
                let score = self.calculate_match_score(resource, request);
                (resource.clone(), score)
            })
            .collect();

        // Sort by score (highest first)
        scored_resources.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        debug!(
            "Top resource scores: {:?}",
            scored_resources
                .iter()
                .take(3)
                .map(|(r, s)| (r.name.clone(), *s))
                .collect::<Vec<_>>()
        );

        Ok(scored_resources
            .into_iter()
            .next()
            .map(|(resource, _)| resource))
    }

    fn calculate_match_score(&self, resource: &Resource, request: &AllocationRequest) -> f64 {
        match self {
            VmAllocationStrategy::PlatformAware => self.platform_score(resource, request) * 10.0,
            VmAllocationStrategy::ResourceEfficient => {
                self.platform_score(resource, request) * 5.0
                    + self.resource_efficiency_score(resource, request) * 8.0
            }
            VmAllocationStrategy::NetworkAware => {
                self.platform_score(resource, request) * 6.0
                    + self.network_score(resource, request) * 7.0
            }
            VmAllocationStrategy::SnapshotAware => {
                self.platform_score(resource, request) * 5.0
                    + self.snapshot_score(resource, request) * 8.0
            }
            VmAllocationStrategy::Weighted {
                platform_weight,
                resource_weight,
                network_weight,
                snapshot_weight,
            } => {
                self.platform_score(resource, request) * platform_weight
                    + self.resource_efficiency_score(resource, request) * resource_weight
                    + self.network_score(resource, request) * network_weight
                    + self.snapshot_score(resource, request) * snapshot_weight
            }
        }
    }

    fn name(&self) -> &'static str {
        match self {
            VmAllocationStrategy::PlatformAware => "vm_platform_aware",
            VmAllocationStrategy::ResourceEfficient => "vm_resource_efficient",
            VmAllocationStrategy::NetworkAware => "vm_network_aware",
            VmAllocationStrategy::SnapshotAware => "vm_snapshot_aware",
            VmAllocationStrategy::Weighted { .. } => "vm_weighted",
        }
    }
}

/// VM allocation preferences for fine-tuning selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmAllocationPreferences {
    /// Preferred platform.
    pub preferred_platform: Option<MachinePlatform>,
    /// Prefer VMs with snapshots.
    pub prefer_snapshots: bool,
    /// Preferred network interface.
    pub preferred_interface: Option<String>,
    /// Maximum acceptable resource overhead (as percentage).
    pub max_resource_overhead: Option<f64>,
    /// Preferred VM location/region tags.
    pub preferred_location_tags: Vec<String>,
}

impl Default for VmAllocationPreferences {
    fn default() -> Self {
        Self {
            preferred_platform: None,
            prefer_snapshots: false,
            preferred_interface: None,
            max_resource_overhead: Some(0.5), // 50% overhead acceptable
            preferred_location_tags: Vec::new(),
        }
    }
}

impl VmAllocationPreferences {
    /// Create preferences for a specific platform.
    pub fn for_platform(platform: MachinePlatform) -> Self {
        Self {
            preferred_platform: Some(platform),
            ..Default::default()
        }
    }

    /// Create preferences that prefer snapshot-enabled VMs.
    pub fn prefer_snapshots() -> Self {
        Self {
            prefer_snapshots: true,
            ..Default::default()
        }
    }

    /// Apply preferences to enhance an allocation request.
    pub fn apply_to_request(&self, request: &mut AllocationRequest) {
        if let Some(platform) = &self.preferred_platform {
            request.constraints.platform = Some(platform.clone());
        }

        if self.prefer_snapshots {
            request
                .constraints
                .custom
                .insert("prefer_snapshots".to_string(), "true".to_string());
        }

        if let Some(interface) = &self.preferred_interface {
            request
                .constraints
                .custom
                .insert("preferred_interface".to_string(), interface.clone());
        }

        // Add location preferences as required tags
        for tag in &self.preferred_location_tags {
            if !request.constraints.required_tags.contains(tag) {
                request.constraints.required_tags.push(tag.clone());
            }
        }
    }
}
