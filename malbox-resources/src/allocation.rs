use crate::types::{Resource, ResourceConstraints, ResourceId, ResourceKind};
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

/// Request for resource allocation.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct AllocationRequest {
    /// Unique request identifier.
    pub request_id: Uuid,
    /// Requesting task ID.
    pub task_id: i32,
    /// Type of resource requested.
    pub resource_kind: ResourceKind,
    /// Resource constraints.
    #[builder(default)]
    pub constraints: ResourceConstraints,
    /// Allocation preferences.
    #[builder(default)]
    pub preferences: AllocationPreferences,
    /// Request priority (higher = more urgent)
    #[builder(default = 5)]
    pub priority: u8,
    /// Request timeout in seconds.
    #[builder(default = 300)]
    pub timeout_seconds: u64,
    /// When the request was created.
    #[builder(default = OffsetDateTime::now_utc())]
    pub created_at: OffsetDateTime,
}

impl AllocationRequest {
    /// Create a new allocation request.
    pub fn new(task_id: String, resource_kind: ResourceKind) -> Self {
        Self::builder()
            .request_id(uuid::Uuid::new_v4().to_string())
            .task_id(task_id)
            .resource_kind(resource_kind)
            .build()
    }

    /// Check if the request has timed out.
    pub fn has_timed_out(&self) -> bool {
        let elapsed = OffsetDateTime::now_utc()
            .unix_timestamp()
            .saturating_sub(self.created_at.unix_timestamp()) as u64;
        elapsed > self.timeout_seconds
    }
}

/// Allocation preferences for resource selection.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct AllocationPreferences {
    /// Prefer resources with specific tags.
    #[builder(default)]
    pub preferred_tags: Vec<String>,
    /// Prefer specific resource by ID.
    pub preferred_resource_id: Option<ResourceId>,
    /// Prefer newer resources.
    #[builder(default = false)]
    pub prefer_newer: bool,
    /// Allow resource provisioning if none available.
    #[builder(default = true)]
    pub allow_provisioning: bool,
    /// Maximum wait time for provisioning in seconds.
    #[builder(default = 600)]
    pub max_provision_wait: u64,
}

impl Default for AllocationPreferences {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Result of a resource allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationResult {
    /// The allocated resource.
    pub resource: Resource,
    /// How the resource was obtained.
    pub allocation_method: AllocationMethod,
    /// Time taken to allocate.
    pub allocation_time_ms: u64,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

/// Method used to obtain the resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationMethod {
    /// Used an existing available resource.
    ExistingResource,
    /// Provisioned a new resource.
    NewlyProvisioned,
    /// Waited for a resource to become available.
    WaitedForAvailability,
}

/// Allocation policy for resource selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationPolicy {
    /// First available resource that meets constraints.
    FirstAvailable,
    /// Best fit based on resource utilization.
    BestFit,
    /// Least recently used resource.
    LeastRecentlyUsed,
    /// Resource with highest matching score.
    HighestScore,
    /// Round robin allocation.
    RoundRobin,
}

impl Default for AllocationPolicy {
    fn default() -> Self {
        AllocationPolicy::FirstAvailable
    }
}

/// Represents an allocated resource with its allocation context.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ResourceAllocation {
    /// The allocated resource.
    pub resource: Resource,
    /// Task ID that owns this allocation.
    pub task_id: String,
    /// When the allocation was made.
    pub allocated_at: OffsetDateTime,
    /// Allocation request that led to this allocation.
    pub request: Option<AllocationRequest>,
    /// Allocation metadata.
    #[builder(default)]
    pub metadata: HashMap<String, String>,
}

impl ResourceAllocation {
    /// Create a new allocation.
    pub fn new(resource: Resource, task_id: String) -> Self {
        Self::builder()
            .resource(resource)
            .task_id(task_id)
            .allocated_at(OffsetDateTime::now_utc())
            .build()
    }

    /// Get the resource ID.
    pub fn resource_id(&self) -> &ResourceId {
        &self.resource.id
    }

    /// Get allocation age in seconds.
    pub fn age_seconds(&self) -> u64 {
        (OffsetDateTime::now_utc().unix_timestamp() - self.allocated_at.unix_timestamp()) as u64
    }

    /// Check if allocation is stale (older than threshold).
    pub fn is_stale(&self, threshold_seconds: u64) -> bool {
        self.age_seconds() > threshold_seconds
    }
}

/// Trait for resource allocation strategies.
#[async_trait::async_trait]
pub trait AllocationStrategy: Send + Sync {
    /// Find the best resource for the given request.
    async fn select_resource(
        &self,
        request: &AllocationRequest,
        available_resources: &[Resource],
    ) -> crate::Result<Option<Resource>>;

    /// Calculate a score for how well a resource matches a request.
    fn calculate_match_score(&self, resource: &Resource, request: &AllocationRequest) -> f64;

    /// Get the strategy name.
    fn name(&self) -> &'static str;
}

/// Default allocation strategy implementation.
pub struct DefaultAllocationStrategy {
    policy: AllocationPolicy,
}

impl DefaultAllocationStrategy {
    /// Create a new strategy with the given policy.
    pub fn new(policy: AllocationPolicy) -> Self {
        Self { policy }
    }
}

#[async_trait::async_trait]
impl AllocationStrategy for DefaultAllocationStrategy {
    async fn select_resource(
        &self,
        request: &AllocationRequest,
        available_resources: &[Resource],
    ) -> crate::Result<Option<Resource>> {
        let mut candidates: Vec<_> = available_resources
            .iter()
            .filter(|r| {
                r.kind == request.resource_kind
                    && r.is_available()
                    && request.constraints.satisfied_by(r)
            })
            .cloned()
            .collect();

        if candidates.is_empty() {
            return Ok(None);
        }

        match self.policy {
            AllocationPolicy::FirstAvailable => Ok(candidates.into_iter().next()),
            AllocationPolicy::BestFit => {
                // Sort by resource utilization (prefer fuller resources)
                candidates.sort_by(|a, b| {
                    let a_score = self.calculate_match_score(a, request);
                    let b_score = self.calculate_match_score(b, request);
                    b_score
                        .partial_cmp(&a_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                Ok(candidates.into_iter().next())
            }
            AllocationPolicy::LeastRecentlyUsed => {
                candidates.sort_by_key(|r| r.updated_at);
                Ok(candidates.into_iter().next())
            }
            AllocationPolicy::HighestScore => {
                candidates.sort_by(|a, b| {
                    let a_score = self.calculate_match_score(a, request);
                    let b_score = self.calculate_match_score(b, request);
                    b_score
                        .partial_cmp(&a_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                Ok(candidates.into_iter().next())
            }
            AllocationPolicy::RoundRobin => {
                // Simple round-robin based on resource creation time
                candidates.sort_by_key(|r| r.created_at);
                Ok(candidates.into_iter().next())
            }
        }
    }

    fn calculate_match_score(&self, resource: &Resource, request: &AllocationRequest) -> f64 {
        let mut score = 0.0;

        // Base score for matching resource kind
        if resource.kind == request.resource_kind {
            score += 10.0;
        }

        // Score for preferred tags
        for preferred_tag in &request.preferences.preferred_tags {
            if resource.tags().contains(preferred_tag) {
                score += 5.0;
            }
        }

        // Score for preferred resource ID
        if let Some(preferred_id) = &request.preferences.preferred_resource_id {
            if &resource.id == preferred_id {
                score += 20.0;
            }
        }

        // Score for resource age if prefer_newer is set
        if request.preferences.prefer_newer {
            let age_hours = (OffsetDateTime::now_utc().unix_timestamp()
                - resource.created_at.unix_timestamp()) as f64
                / 3600.0;
            score += (24.0 - age_hours.min(24.0)).max(0.0);
        }

        // Score for resource health
        if resource.status.healthy {
            score += 5.0;
        }

        score
    }

    fn name(&self) -> &'static str {
        match self.policy {
            AllocationPolicy::FirstAvailable => "first_available",
            AllocationPolicy::BestFit => "best_fit",
            AllocationPolicy::LeastRecentlyUsed => "lru",
            AllocationPolicy::HighestScore => "highest_score",
            AllocationPolicy::RoundRobin => "round_robin",
        }
    }
}
