use crate::{
    command::AsyncCommand,
    parser::terraform::parse_variables,
    terraform::{state::StateManager, types::WorkspaceConfig, workspace::WorkspaceManager},
    types::Platform,
    Error, Result,
};
use bon::{bon, Builder};
use malbox_config::{machinery::MachineProvider, Config, PathConfig};
use malbox_database::repositories::machinery::{
    insert_machine, Machine, MachineArch, MachinePlatform,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};

pub struct VmConfig {
    pub name: String,
    pub platform: MachinePlatform,
    pub memory: u32,
    pub cpus: u32,
    pub disk_size: u32,
    pub snapshot: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VmInstance {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub platform: MachinePlatform,
    pub interface: Option<String>,
    pub snapshot: Option<String>,
}

pub struct TerraformManager {
    config: Config,
    workspace_manager: WorkspaceManager,
    state_manager: StateManager,
    infrastructure_dir: PathBuf,
    db_pool: malbox_database::PgPool,
}

#[bon]
impl TerraformManager {
    #[builder]
    pub fn new(db_pool: malbox_database::PgPool, config: Config) -> Self {
        let workspace_manager = WorkspaceManager::new(config.clone());
        let state_manager = StateManager::new(config.clone());
        let infrastructure_dir = config.paths.terraform_dir.clone();

        Self {
            config,
            workspace_manager,
            state_manager,
            infrastructure_dir,
            db_pool,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        let tf_check = AsyncCommand::new("terraform").arg("version").run().await?;

        if tf_check.success() {
            let version = String::from(
                tf_check
                    .stdout()
                    .lines()
                    .next()
                    .unwrap_or("unknown version"),
            );
            debug!("Terraform installed: {}", version);
        } else {
            return Err(
                Error::Terraform(
                    "Terraform not found or not executable. Make sure you have properly installed Terraform.".to_string()
                )
            );
        }

        if !self.infrastructure_dir.exists() {
            return Err(Error::Terraform(format!(
                "Infrastructure directory not found: {:?}",
                self.infrastructure_dir
            )));
        }

        Ok(())
    }

    // NOTE: async? worth it here?
    fn create_workspace_config(
        &self,
        env_name: &str,
        auto_approve: bool,
    ) -> Result<WorkspaceConfig> {
        let env_dir = self
            .infrastructure_dir
            .join("environments")
            .join("env_name");

        if !env_dir.exists() {
            return Err(Error::Terraform(format!(
                "Environment directory not found: {:?}",
                env_dir
            )));
        }

        let workspace = env_name.to_string();
        let mut variables = HashMap::new();

        variables.extend(self.config.machinery.terraform.variables.clone());

        let env_vars_file = env_dir.join("terraform.tfvars");
        if env_vars_file.exists() {
            let vars_content = std::fs::read_to_string(env_vars_file)?;
            let file_variables = parse_variables(&vars_content)?;
            variables.extend(file_variables);
        }

        Ok(WorkspaceConfig {
            name: env_name.to_string(),
            working_dir: env_dir,
            workspace,
            variables,
            backend_config: self.config.machinery.terraform.backend_config.clone(),
            target: None,
            auto_approve,
        })
    }

    pub async fn provision_vm(&self, vm_config: &VmConfig) -> Result<VmInstance> {
        let env_name = match vm_config.platform {
            MachinePlatform::Windows => "windows",
            MachinePlatform::Linux => "linux",
            _ => "default",
        };

        let mut workspace_config = self.create_workspace_config(env_name, true)?;

        workspace_config
            .variables
            .insert("vm_name".to_string(), vm_config.name.clone());
        workspace_config
            .variables
            .insert("memory".to_string(), vm_config.memory.to_string());
        workspace_config
            .variables
            .insert("cpus".to_string(), vm_config.cpus.to_string());
        workspace_config
            .variables
            .insert("disk_size".to_string(), vm_config.disk_size.to_string());

        if let Some(snapshot) = &vm_config.snapshot {
            workspace_config
                .variables
                .insert("snapshot".to_string(), snapshot.clone());
        }

        workspace_config.target = Some(format!("module.vm.{}", vm_config.name));

        info!("Provisioning VM '{}' using Terraform", vm_config.name);
        self.workspace_manager.apply(&workspace_config).await?;

        // TODO: Actually extract VM info from terraform state
        // NOTE: We could already get information as IP from config,
        // but the user could override it via tfvars or terraform template
        // hence, it may be worth ignoring it, and just extracting it from
        // terraform state, as we would do with ID

        let state_output = self.state_manager.show(&workspace_config).await?;

        let vm_instance = VmInstance {
            id: "1234".to_string(),
            name: vm_config.name.clone(),
            platform: vm_config.platform.clone(),
            ip: "10.10.10.10".to_string(),
            interface: Some("eth0".to_string()),
            snapshot: vm_config.snapshot.clone(),
        };

        info!(
            "VM '{}' provisioned succesfully with IP {}",
            vm_instance.name, vm_instance.ip
        );

        self.register_vm_in_database(&vm_instance).await?;

        Ok(vm_instance)
    }

    pub async fn destroy_vm(&self, vm_name: &str, platform: MachinePlatform) -> Result<()> {
        let env_name = match platform {
            MachinePlatform::Windows => "windows",
            MachinePlatform::Linux => "linux",
            _ => "default",
        };

        let mut workspace_config = self.create_workspace_config(env_name, true)?;

        workspace_config.target = Some(format!("module.vm.{}", vm_name));

        info!("Destroying VM '{}'", vm_name);
        self.workspace_manager.destroy(&workspace_config).await?;

        // TODO:
        // Remove VM from DB

        Ok(())
    }

    async fn register_vm_in_database(&self, vm: &VmInstance) -> Result<()> {
        let machine = Machine {
            id: None,
            name: vm.name.clone(),
            label: vm.name.clone(),
            arch: MachineArch::X64,
            platform: vm.platform.clone(),
            ip: vm.ip.clone(),
            tags: None,
            interface: vm.interface.clone(),
            snapshot: vm.snapshot.clone(),
            locked: false,
            locked_changed_on: None,
            status: Some("ready".to_string()),
            status_changed_on: None,
            reserved: false,
        };

        insert_machine(&self.db_pool, machine).await?;

        Ok(())
    }
}
