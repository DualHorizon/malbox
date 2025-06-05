use crate::{
    allocation::AllocationRequest,
    error::{ResourceError, Result},
    types::{Resource, ResourceId, ResourceKind, ResourceSpec, ResourceState, ResourceStatus},
};
use bon::Builder;
use malbox_config::Config;
use malbox_database::{
    repositories::machinery::{
        fetch_machine, fetch_machines, insert_machine, lock_machine, unlock_machine, Machine,
        MachineArch, MachineFilter, MachinePlatform,
    },
    PgPool,
};
use malbox_terraform::manager::{TerraformManager, VmConfig, VmInstance};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::{OffsetDateTime, PrimitiveDateTime};
use tracing::{debug, error, info, warn};

/// VM-specific resource specification.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct VmSpec {
    /// VM name.
    pub name: String,
    /// Target platform.
    pub platform: MachinePlatform,
    /// CPU cores.
    #[builder(default = 2)]
    pub cpu_cores: u32,
    /// Memory in MB.
    #[builder(default = 4096)]
    pub memory_mb: u32,
    /// Disk size in GB.
    #[builder(default = 100)]
    pub disk_size_gb: u32,
    /// Network interface.
    pub interface: Option<String>,
    /// VM snapshot to use.
    pub snapshot: Option<String>,
    /// Additional tags.
    #[builder(default)]
    pub tags: Vec<String>,
}

/// Specialized resource wrapper for VMs.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct VmResource {
    /// Base resource information.
    pub resource: Resource,
    /// VM-specific specification.
    pub vm_spec: VmSpec,
    /// Database machine ID.
    pub machine_id: Option<i32>,
    /// IP address.
    pub ip_address: Option<String>,
    /// Terraform instance info.
    pub terraform_instance: Option<VmInstance>,
}

impl VmResource {
    /// Create from a base resource and VM spec.
    pub fn new(resource: Resource, vm_spec: VmSpec) -> Self {
        Self::builder().resource(resource).vm_spec(vm_spec).build()
    }

    /// Get the VM's IP address.
    pub fn ip_address(&self) -> Option<&str> {
        self.ip_address.as_deref()
    }

    /// Check if VM is ready for use.
    pub fn is_ready(&self) -> bool {
        matches!(
            self.resource.status.state,
            ResourceState::Available | ResourceState::Stopped
        ) && self.ip_address.is_some()
    }

    /// Convert to base Resource.
    pub fn into_resource(self) -> Resource {
        self.resource
    }
}

/// Manager for VM resources.
pub struct VmManager {
    /// Database connection pool.
    db_pool: PgPool,
    /// Malbox configuration.
    config: Config,
    /// Terraform manager for provisioning.
    terraform_manager: TerraformManager,
}

impl VmManager {
    /// Create a new VM manager.
    pub async fn new(db_pool: PgPool, config: Config) -> Result<Self> {
        let terraform_manager = TerraformManager::new(db_pool.clone(), config.clone());

        // Initialize Terraform
        terraform_manager
            .initialize()
            .await
            .map_err(|e| ResourceError::ProvisioningFailed {
                details: e.to_string(),
            })?;

        Ok(Self {
            db_pool,
            config,
            terraform_manager,
        })
    }

    /// Create a new VM resource.
    pub async fn create_resource(&self, spec: ResourceSpec) -> Result<Resource> {
        if spec.kind != ResourceKind::VirtualMachine {
            return Err(ResourceError::Configuration {
                message: format!("Expected VirtualMachine resource, got {}", spec.kind),
            });
        }

        // Extract VM-specific configuration
        let vm_spec = self.extract_vm_spec_from_resource_spec(&spec)?;

        info!("Creating VM resource: {}", spec.name);

        // Create the base resource
        let mut resource = Resource::from_spec(spec);

        // Add VM-specific properties
        resource
            .properties
            .insert("platform".to_string(), vm_spec.platform.to_string());
        resource
            .properties
            .insert("cpu_cores".to_string(), vm_spec.cpu_cores.to_string());
        resource
            .properties
            .insert("memory_mb".to_string(), vm_spec.memory_mb.to_string());
        resource
            .properties
            .insert("disk_size_gb".to_string(), vm_spec.disk_size_gb.to_string());

        if let Some(interface) = &vm_spec.interface {
            resource
                .properties
                .insert("interface".to_string(), interface.clone());
        }

        if let Some(snapshot) = &vm_spec.snapshot {
            resource
                .properties
                .insert("snapshot".to_string(), snapshot.clone());
        }

        // Provision the VM via Terraform
        let vm_config = VmConfig {
            name: vm_spec.name.clone(),
            platform: vm_spec.platform,
            memory: vm_spec.memory_mb,
            cpus: vm_spec.cpu_cores,
            disk_size: vm_spec.disk_size_gb,
            snapshot: vm_spec.snapshot.clone(),
        };

        let vm_instance = self
            .terraform_manager
            .provision_vm(&vm_config)
            .await
            .map_err(|e| ResourceError::ProvisioningFailed {
                details: e.to_string(),
            })?;

        // Create database record
        let machine = Machine {
            id: None,
            name: vm_spec.name.clone(),
            label: resource.name.clone(),
            arch: MachineArch::X64,
            platform: vm_spec.platform,
            ip: vm_instance.ip.clone(),
            interface: vm_instance.interface.clone(),
            tags: Some(vm_spec.tags.clone()),
            snapshot: vm_instance.snapshot.clone(),
            locked: false,
            locked_changed_on: None,
            status: Some("provisioned".to_string()),
            status_changed_on: Some(PrimitiveDateTime::now()),
            reserved: false,
        };

        let db_machine = insert_machine(&self.db_pool, machine)
            .await
            .map_err(ResourceError::Database)?;

        // Update resource with provisioned information
        resource
            .properties
            .insert("ip_address".to_string(), vm_instance.ip.clone());
        resource
            .properties
            .insert("machine_id".to_string(), db_machine.id.unwrap().to_string());
        resource.update_status(ResourceState::Available);

        info!(
            "Successfully created VM resource: {} with IP {}",
            resource.id, vm_instance.ip
        );
        Ok(resource)
    }

    /// Get a VM resource by ID.
    pub async fn get_resource(&self, id: &ResourceId) -> Result<Option<Resource>> {
        // Try to find by machine name (using resource ID as name)
        let machine_filter = MachineFilter::builder().label(id.to_string()).build();

        if let Some(machine) = fetch_machine(&self.db_pool, Some(machine_filter))
            .await
            .map_err(ResourceError::Database)?
        {
            let resource = self.machine_to_resource(&machine)?;
            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    /// List all VM resources.
    pub async fn list_resources(&self) -> Result<Vec<Resource>> {
        let machines = fetch_machines(&self.db_pool, None)
            .await
            .map_err(ResourceError::Database)?;

        let mut resources = Vec::new();
        for machine in machines {
            match self.machine_to_resource(&machine) {
                Ok(resource) => resources.push(resource),
                Err(e) => {
                    warn!(
                        "Failed to convert machine {} to resource: {}",
                        machine.name, e
                    );
                }
            }
        }

        Ok(resources)
    }

    /// Update a VM resource.
    pub async fn update_resource(&self, resource: &Resource) -> Result<()> {
        if resource.kind != ResourceKind::VirtualMachine {
            return Err(ResourceError::Configuration {
                message: "Resource is not a VM".to_string(),
            });
        }

        // Extract machine ID from properties
        let machine_id: i32 = resource
            .properties
            .get("machine_id")
            .ok_or_else(|| ResourceError::Configuration {
                message: "VM resource missing machine_id".to_string(),
            })?
            .parse()
            .map_err(|_| ResourceError::Configuration {
                message: "Invalid machine_id format".to_string(),
            })?;

        // Update lock status based on allocation
        if resource.is_allocated() {
            lock_machine(&self.db_pool, machine_id, Some("allocated"))
                .await
                .map_err(ResourceError::Database)?;
        } else {
            unlock_machine(&self.db_pool, machine_id)
                .await
                .map_err(ResourceError::Database)?;
        }

        debug!("Updated VM resource: {}", resource.id);
        Ok(())
    }

    /// Delete a VM resource.
    pub async fn delete_resource(&self, id: &ResourceId) -> Result<()> {
        info!("Deleting VM resource: {}", id);

        // Get the resource to extract information
        let resource = self
            .get_resource(id)
            .await?
            .ok_or_else(|| ResourceError::NotFound { id: id.to_string() })?;

        // Extract platform and name for Terraform cleanup
        let platform = resource
            .platform()
            .ok_or_else(|| ResourceError::Configuration {
                message: "VM resource missing platform".to_string(),
            })?;

        // Destroy via Terraform
        self.terraform_manager
            .destroy_vm(&resource.name, platform.clone())
            .await
            .map_err(|e| ResourceError::ProvisioningFailed {
                details: e.to_string(),
            })?;

        // TODO: Remove from database

        info!("Successfully deleted VM resource: {}", id);
        Ok(())
    }

    /// Provision a VM for an allocation request.
    pub async fn provision_for_request(&self, request: &AllocationRequest) -> Result<Resource> {
        info!(
            "Provisioning VM for allocation request from task: {}",
            request.task_id
        );

        // Determine platform from constraints
        let platform = request
            .constraints
            .platform
            .unwrap_or(MachinePlatform::Linux);

        // Create VM spec from constraints
        let vm_spec = VmSpec::builder()
            .name(format!("vm-{}-{}", request.task_id, uuid::Uuid::new_v4()))
            .platform(platform)
            .cpu_cores(request.constraints.min_cpu_cores.unwrap_or(2))
            .memory_mb(request.constraints.min_memory_mb.unwrap_or(4096))
            .tags(request.constraints.required_tags.clone())
            .build();

        // Create resource spec
        let resource_spec = ResourceSpec::builder()
            .name(vm_spec.name.clone())
            .kind(ResourceKind::VirtualMachine)
            .tags(vm_spec.tags.clone())
            .build();

        // Create the resource
        self.create_resource(resource_spec).await
    }

    /// Convert a database Machine to a Resource.
    fn machine_to_resource(&self, machine: &Machine) -> Result<Resource> {
        let mut properties = HashMap::new();
        properties.insert("platform".to_string(), format!("{:?}", machine.platform));
        properties.insert("ip_address".to_string(), machine.ip.clone());
        properties.insert("machine_id".to_string(), machine.id.unwrap().to_string());

        if let Some(interface) = &machine.interface {
            properties.insert("interface".to_string(), interface.clone());
        }

        if let Some(snapshot) = &machine.snapshot {
            properties.insert("snapshot".to_string(), snapshot.clone());
        }

        // Determine resource state from machine status
        let state = if machine.locked {
            ResourceState::Allocated
        } else if machine.status.as_deref() == Some("provisioned") {
            ResourceState::Available
        } else {
            ResourceState::Provisioning
        };

        let status = ResourceStatus::new(state);

        let now = OffsetDateTime::now_utc();
        let resource = Resource::builder()
            .id(ResourceId::new()) // Generate new ID for resource
            .name(machine.label.clone())
            .kind(ResourceKind::VirtualMachine)
            .status(status)
            .properties(properties)
            .tags(machine.tags.clone().unwrap_or_default())
            .created_at(now)
            .updated_at(now)
            .allocated_to(if machine.locked {
                "unknown".to_string()
            } else {
                None
            })
            .build();

        Ok(resource)
    }

    /// Extract VM specification from resource specification.
    fn extract_vm_spec_from_resource_spec(&self, spec: &ResourceSpec) -> Result<VmSpec> {
        let platform = spec.constraints.platform.unwrap_or(MachinePlatform::Linux);

        let cpu_cores = spec.constraints.min_cpu_cores.unwrap_or(2);
        let memory_mb = spec.constraints.min_memory_mb.unwrap_or(4096);

        // Extract other properties from config
        let disk_size_gb = spec
            .config
            .get("disk_size_gb")
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        let interface = spec.config.get("interface").cloned();
        let snapshot = spec.config.get("snapshot").cloned();

        Ok(VmSpec::builder()
            .name(spec.name.clone())
            .platform(platform)
            .cpu_cores(cpu_cores)
            .memory_mb(memory_mb)
            .disk_size_gb(disk_size_gb)
            .interface(interface)
            .snapshot(snapshot)
            .tags(spec.tags.clone())
            .build())
    }
}
