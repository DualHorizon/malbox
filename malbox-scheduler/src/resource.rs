use malbox_config::Config;
use malbox_database::{
    repositories::machinery::{
        fetch_machine, fetch_machines, lock_machine, unlock_machine, Machine, MachineFilter,
        MachinePlatform,
    },
    PgPool,
};
use malbox_infra::terraform::manager::{TerraformManager, VmConfig};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("No suitable VM available")]
    NoSuitableVM,
    #[error("Failed to allocate resources: {0}")]
    AllocationFailed(String),
    #[error("Database error: {0}")]
    Database(#[from] malbox_database::error::DatabaseError),
    #[error("Terraform error: {0}")]
    Terraform(String),
    #[error("VM operation failed: {0}")]
    VMOperation(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
}

type Result<T> = std::result::Result<T, ResourceError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    VM,
    Network,
    Storage,
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub id: String,
    pub kind: ResourceKind,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub allocated: bool,
    pub task_id: Option<String>,
}

impl Resource {
    pub fn from_machine(machine: &Machine) -> Self {
        let mut properties = HashMap::new();
        properties.insert("platform".to_string(), format!("{:?}", machine.platform));
        properties.insert("ip".to_string(), machine.ip.clone());

        if let Some(snapshot) = &machine.snapshot {
            properties.insert("snapshot".to_string(), snapshot.clone());
        }

        if let Some(interface) = &machine.interface {
            properties.insert("interface".to_string(), interface.clone());
        }

        Self {
            id: machine.id.to_string(),
            kind: ResourceKind::VM,
            name: machine.name.clone(),
            properties,
            allocated: machine.locked,
            task_id: None,
        }
    }

    pub fn platform(&self) -> Option<MachinePlatform> {
        self.properties
            .get("platform")
            .and_then(|p| match p.as_str() {
                "Windows" => Some(MachinePlatform::Windows),
                "Linux" => Some(MachinePlatform::Linux),
                _ => None,
            })
    }

    pub fn ip(&self) -> Option<&str> {
        self.properties.get("ip").map(|s| s.as_str())
    }

    pub fn snapshot(&self) -> Option<&str> {
        self.properties.get("snapshot").map(|s| s.as_str())
    }
}

pub struct ResourceManager {
    db: PgPool,
    config: Config,
    resources: RwLock<HashMap<String, Resource>>,
    allocations: RwLock<HashMap<String, HashSet<String>>>,
    terraform_manager: Arc<TerraformManager>,
}

impl ResourceManager {
    pub fn new(db: PgPool, config: Config) -> Self {
        let terraform_manager = Arc::new(
            TerraformManager::builder()
                .db_pool(db.clone())
                .config(config.clone())
                .build(),
        );

        Self {
            db,
            config,
            resources: RwLock::new(HashMap::new()),
            allocations: RwLock::new(HashMap::new()),
            terraform_manager,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        self.load_resources().await?;

        self.terraform_manager
            .initialize()
            .await
            .map_err(|e| ResourceError::Terraform(e.to_string()))?;

        Ok(())
    }

    async fn load_resources(&self) -> Result<()> {
        let machines = fetch_machines(&self.db, None).await?;

        let mut resources = self.resources.write().await;
        for machine in machines {
            let resource = Resource::from_machine(&machine);
            resources.insert(resource.id.clone(), resource);
        }

        info!("Loaded {} resources from database", resources.len());
        Ok(())
    }

    pub async fn allocate_vm_for_task(
        &self,
        task_id: i32,
        platform: Option<MachinePlatform>,
        specific_machine: Option<&str>,
    ) -> Result<Resource> {
        {
            let allocations = self.allocations.read().await;
            if let Some(resource_ids) = allocations.get(&task_id.to_string()) {
                for resource_id in resource_ids {
                    let resources = self.resources.read().await;
                    if let Some(resource) = resources.get(resource_id) {
                        if resource.kind == ResourceKind::VM {
                            return Ok(resource.clone());
                        }
                    }
                }
            }
        }

        let vm = if let Some(machine_name) = specific_machine {
            self.allocate_specific_machine(&task_id.to_string(), machine_name)
                .await?
        } else {
            self.allocate_suitable_machine(&task_id.to_string(), platform)
                .await?
        };

        {
            let mut allocations = self.allocations.write().await;
            allocations
                .entry(task_id.to_string())
                .or_insert_with(HashSet::new)
                .insert(vm.id.clone());
        }

        Ok(vm)
    }

    async fn allocate_specific_machine(
        &self,
        task_id: &str,
        machine_name: &str,
    ) -> Result<Resource> {
        let machine_filter = MachineFilter::builder()
            .label(machine_name.to_string())
            .locked(false)
            .build();

        let machine = fetch_machine(&self.db, Some(machine_filter))
            .await?
            .ok_or_else(|| {
                ResourceError::NotFound(format!("Machine not found: {}", machine_name))
            })?;

        lock_machine(&self.db, machine.id.unwrap(), None).await?;

        let mut resource = Resource::from_machine(&machine);
        resource.allocated = true;
        resource.task_id = Some(task_id.to_string());

        {
            let mut resources = self.resources.write().await;
            resources.insert(resource.id.clone(), resource.clone());
        }

        info!(
            "Allocated specific machine '{}' for task '{}'",
            machine_name, task_id
        );
        Ok(resource)
    }

    async fn allocate_suitable_machine(
        &self,
        task_id: &str,
        platform: Option<MachinePlatform>,
    ) -> Result<Resource> {
        let machine_filter = MachineFilter::builder()
            .locked(false)
            .maybe_platform(platform.clone())
            .build();

        let machine = fetch_machine(&self.db, Some(machine_filter)).await?;

        if let Some(machine) = machine {
            lock_machine(&self.db, machine.id.unwrap(), None).await?;

            let mut resource = Resource::from_machine(&machine);
            resource.allocated = true;
            resource.task_id = Some(task_id.to_string());

            {
                let mut resources = self.resources.write().await;
                resources.insert(resource.id.clone(), resource.clone());
            }

            info!(
                "Allocated machine '{}' for task '{}'",
                machine.name, task_id
            );
            return Ok(resource);
        }

        let platform = platform.unwrap_or(MachinePlatform::Windows);

        info!(
            "No available machine found, provisioning new {:?} VM for task '{}'",
            platform, task_id
        );

        let vm_config = VmConfig {
            name: format!("vm-{:?}-{}", platform, task_id),
            platform,
            memory: 4096,
            cpus: 2,
            disk_size: 100,
            snapshot: None,
        };

        let vm = self
            .terraform_manager
            .provision_vm(&vm_config)
            .await
            .map_err(|e| ResourceError::Terraform(e.to_string()))?;

        let mut properties = HashMap::new();
        properties.insert("platform".to_string(), format!("{:?}", vm.platform));
        properties.insert("ip".to_string(), vm.ip.clone());

        if let Some(snapshot) = &vm.snapshot {
            properties.insert("snapshot".to_string(), snapshot.clone());
        }

        if let Some(interface) = &vm.interface {
            properties.insert("interface".to_string(), interface.clone());
        }

        let resource = Resource {
            id: vm.id.clone(),
            kind: ResourceKind::VM,
            name: vm.name.clone(),
            properties,
            allocated: true,
            task_id: Some(task_id.to_string()),
        };

        {
            let mut resources = self.resources.write().await;
            resources.insert(resource.id.clone(), resource.clone());
        }

        info!(
            "Provisioned new VM '{}' for task '{}'",
            resource.name, task_id
        );
        Ok(resource)
    }

    pub async fn release_resources(&self, task_id: i32) -> Result<()> {
        let resource_ids = {
            let mut allocations = self.allocations.write().await;
            allocations.remove(&task_id.to_string()).unwrap_or_default()
        };

        for resource_id in resource_ids {
            let mut resources = self.resources.write().await;
            if let Some(resource) = resources.get_mut(&resource_id) {
                if resource.kind == ResourceKind::VM {
                    unlock_machine(&self.db, resource_id.parse().unwrap_or(0)).await?;

                    resource.allocated = false;
                    resource.task_id = None;

                    info!("Released VM '{}' from task '{}'", resource.name, task_id);
                }
            }
        }

        Ok(())
    }

    pub async fn get_vm_for_task(&self, task_id: &str) -> Result<Option<Resource>> {
        let allocations = self.allocations.read().await;
        if let Some(resource_ids) = allocations.get(task_id) {
            let resources = self.resources.read().await;
            for resource_id in resource_ids {
                if let Some(resource) = resources.get(resource_id) {
                    if resource.kind == ResourceKind::VM {
                        return Ok(Some(resource.clone()));
                    }
                }
            }
        }

        Ok(None)
    }
}
