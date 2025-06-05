use bon::Builder;
use malbox_database::repositories::machinery::MachinePlatform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

/// Unique identifier for a resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub Uuid);

impl ResourceId {
    /// Create a new random resource ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID.
    pub fn uuid(&self) -> Uuid {
        self.0
    }

    /// Convert to string.
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

/// A type of resource.
#[derive(Debug, CLone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    /// Virtual machine resource.
    VirtualMachine,
    /// Network resource.
    Network,
    /// Storage resource.
    Storage,
    /// Generic resource type.
    Generic(String),
}

impl std::fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceKind::VirtualMachine => write!(f, "vm"),
            ResourceKind::Network => write!(f, "network"),
            ResourceKind::Storage => write!(f, "storage"),
            ResourceKind::Generic(name) => write!(f, "{}", name),
        }
    }
}

/// Current state of a resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceState {
    /// Resource is being created/provisioned.
    Provisioning,
    /// Resource is available for allocation.
    Available,
    /// Resource is allocated to a task.
    Allocated,
    /// Resource is running/in use
    InUse,
    /// Resource is being stopped.
    Stopping,
    /// Resource is stopped but not deallocated.
    Stopped,
    /// Resource is being destroyed.
    Destroying,
    /// Resource has been destroyed.
    Destroyed,
    /// Resource is in an erronous state.
    Error(String),
}

/// Resource status information.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ResourceStatus {
    /// Current state.
    pub state: ResourceState,
    /// When the status was last updated.
    pub last_updated: OffsetDateTime,
    /// Optional status message.
    pub message: Option<String>,
    /// Health check status.
    #[builder(default = true)]
    pub healthy: bool,
    /// Additional metadata.
    #[builder(default)]
    pub metadata: HashMap<String, String>,
}

impl ResourceStatus {
    /// Create a new status with the given state.
    pub fn new(state: ResourceState) -> Self {
        Self {
            state,
            last_updated: OffsetDateTime::now_utc(),
            message: None,
            healthy: true,
            metadata: HashMap::new(),
        }
    }

    /// Update the state and timestamp.
    pub fn update_state(&mut self, new_state: ResourceState) {
        self.state = new_state;
        self.last_updated = OffsetDateTime::now_utc();
    }
}

/// Resource constraints and requirements.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ResourceConstraints {
    /// Platform requirements.
    pub platform: Option<MachinePlatform>,
    /// Minimum CPU cores.
    pub min_cpu_cores: Option<u32>,
    /// Minimum memory in MB.
    pub min_memory_mb: Option<u32>,
    /// Required tags.
    #[builder(default)]
    pub required_tags: Vec<String>,
    /// Custom constraints.
    #[builder(default)]
    pub custom: HashMap<String, String>,
}

impl ResourceConstraints {
    /// Create empty constraints
    pub fn empty() -> Self {
        Self {
            platform: None,
            min_cpu_cores: None,
            min_memory_mb: None,
            required_tags: Vec::new(),
            custom: HashMap::new(),
        }
    }

    /// Check if constraints are satisfied by a resource
    pub fn satisfied_by(&self, resource: &Resource) -> bool {
        // Platform check
        if let Some(required_platform) = &self.platform {
            if let Some(resource_platform) = resource.platform() {
                if resource_platform != required_platform {
                    return false;
                }
            } else {
                return false;
            }
        }

        // CPU check
        if let Some(min_cpu) = self.min_cpu_cores {
            if let Some(cpu_cores) = resource.cpu_cores() {
                if cpu_cores < min_cpu {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Memory check
        if let Some(min_memory) = self.min_memory_mb {
            if let Some(memory) = resource.memory_mb() {
                if memory < min_memory {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Tags check
        for required_tag in &self.required_tags {
            if !resource.tags().contains(required_tag) {
                return false;
            }
        }

        true
    }
}

/// Resource specification for creation.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ResourceSpec {
    /// Resource name.
    pub name: String,
    /// Resource kind.
    pub kind: ResourceKind,
    /// Resource constraints.
    #[builder(default)]
    pub constraints: ResourceConstraints,
    /// Platform-specific configuration.
    #[builder(default)]
    pub config: HashMap<String, String>,
    /// Resource tags.
    #[builder(default)]
    pub tags: Vec<String>,
}

/// Representation of a managed resource.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Resource {
    /// Unique resource identifier.
    pub id: ResourceId,
    /// Resource name.
    pub name: String,
    /// Resource kind.
    pub kind: ResourceKind,
    /// Current status.
    pub status: ResourceStatus,
    /// Resource properties.
    #[builder(default)]
    pub properties: HashMap<String, String>,
    /// Resource tags.
    #[builder(default)]
    pub tags: Vec<String>,
    /// When the resource was created.
    pub created_at: OffsetDateTime,
    /// WHen the resource was last updated.
    pub updated_at: OffsetDateTime,
    /// Optional task ID if allocated.
    pub allocated_to: Option<String>,
}

impl Resource {
    /// Create a new resource from a spec.
    pub fn from_spec(spec: ResourceSpec) -> Self {
        let now = OffsetDateTime::now_utc();

        Self {
            id: ResourceId::new(),
            name: spec.name,
            kind: spec.kind,
            status: ResourceStatus::new(ResourceState::Provisioning),
            properties: spec.config,
            tags: spec.tags,
            created_at: now,
            updated_at: now,
            allocated_to: None,
        }
    }

    /// Get the resource's platform if it's a VM.
    pub fn platform(&self) -> Option<&MachinePlatform> {
        self.properties
            .get("platform")
            .and_then(|p| match p.as_str() {
                "windows" => Some(&MachinePlatform::Windows),
                "linux" => Some(&MachinePlatform::Linux),
                _ => None,
            })
    }

    /// Get CPU cores if specified.
    pub fn cpu_cores(&self) -> Option<u32> {
        self.properties
            .get("cpu_cores")
            .and_then(|s| s.parse().ok())
    }

    /// Get memory in MB if specified.
    pub fn memory_mb(&self) -> Option<u32> {
        self.properties
            .get("memory_mb")
            .and_then(|s| s.parse().ok())
    }

    /// Get resource tags.
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Check if resource is available for allocation.
    pub fn is_available(&self) -> bool {
        self.status.state.can_allocate() && self.allocated_to.is_none()
    }

    /// Check if resource is allocated.
    pub fn is_allocated(&self) -> bool {
        self.allocated_to.is_some()
    }

    /// Update resource status.
    pub fn update_status(&mut self, new_state: ResourceState) {
        self.status.update_state(new_state);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Allocate resource to a task.
    pub fn allocate_to(&mut self, task_id: &str) -> crate::Result<()> {
        if !self.is_available() {
            return Err(crate::ResourceError::NotAvailable {
                reason: format!("Resource {} is not available", self.id),
            });
        }

        self.allocated_to = Some(task_id.to_string());
        self.update_status(ResourceState::Allocated);
        Ok(())
    }

    /// Deallocate resource from a task.
    pub fn deallocate(&mut self) {
        self.allocated_to = None;
        self.update_status(ResourceState::Available);
    }
}
