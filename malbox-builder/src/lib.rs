use anyhow::Context;
use malbox_config::templates::{AnsibleConfig, Template, TemplateConfig};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{error, info};

pub mod error;

// NOTE: !!! This implementation is not finished yet !!!
// It is only a basic shim and non-idiomatic code (for some parts), designed to test out stuff
// priorizing functionality over code quality - will refactor later!

#[derive(Debug)]
pub struct Builder {
    working_dir: PathBuf,
    template_config: TemplateConfig,
}

impl Builder {
    pub fn new(template_config: TemplateConfig) -> Self {
        Self {
            working_dir: PathBuf::from("./configuration"),
            template_config,
        }
    }

    pub fn with_working_dir(
        template_config: TemplateConfig,
        working_dir: impl AsRef<Path>,
    ) -> Self {
        Self {
            working_dir: working_dir.as_ref().to_path_buf(),
            template_config,
        }
    }

    fn get_template_path(&self, platform: &str) -> PathBuf {
        self.working_dir
            .canonicalize()
            .unwrap_or(self.working_dir.clone())
            .join("templates")
            .join(platform)
            .join("base.pkr.hcl")
    }

    fn get_builder_name(&self, hypervisor: &str) -> anyhow::Result<&str> {
        match hypervisor {
            "vmware" => Ok("vsphere-iso.windows_analyzer"),
            "virtualbox" => Ok("virtualbox-iso.windows_analyzer"),
            "kvm" => Ok("qemu.windows_analyzer"),
            _ => anyhow::bail!("Unsupported hypervisor type: {}", hypervisor),
        }
    }

    fn get_playbook_path(&self, platform: &str) -> PathBuf {
        self.working_dir
            .join("templates")
            .join(platform)
            .join("playbooks")
            .join("base.yml")
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        info!("Initializing builder environment...");

        let output = Command::new("packer")
            .current_dir(&self.working_dir)
            .arg("init")
            .arg("templates/packer_plugins.pkr.hcl")
            .output()
            .await
            .context("Failed to initialize packer plugins")?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            error!("Failed to initialize packer plugins");
            error!("STDOUT: {}", stdout);
            error!("STDERR: {}", stderr);

            anyhow::bail!(
                "Packer plugin initialization failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
                stdout,
                stderr
            );
        }

        if !output.stdout.is_empty() {
            info!("Packer output: {}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    }

    pub async fn build(
        &self,
        platform: &str,
        name: &str,
        hypervisor: &str,
        force: bool,
    ) -> anyhow::Result<()> {
        let template = self.get_template(platform, name)?;
        let template_path = self.get_template_path(platform);
        let builder_name = self.get_builder_name(hypervisor)?;

        println!("{}", builder_name);

        info!("Building template {} for platform {}", name, platform);

        if !template_path.exists() {
            anyhow::bail!("Packer template not found at {:?}", template_path);
        }

        let mut cmd = Command::new("packer");
        cmd.current_dir(&self.working_dir)
            .arg("build")
            .arg("-timestamp-ui");

        cmd.arg(format!("-only={}", builder_name));

        if force {
            cmd.arg("-force");
        }

        for (key, value) in &template.packer.vars {
            cmd.arg("-var").arg(format!("{}={}", key, value));
        }

        cmd.arg(&template_path);

        let output = cmd.output().await.context("Failed to execute packer")?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            error!("Packer build failed");
            error!("STDOUT: {}", stdout);
            error!("STDERR: {}", stderr);

            anyhow::bail!(
                "Packer build failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
                stdout,
                stderr
            );
        }

        if !output.stdout.is_empty() {
            info!("Packer output: {}", String::from_utf8_lossy(&output.stdout));
        }
        if let Some(ansible_config) = &template.ansible {
            self.run_ansible_provisioning(ansible_config).await?;
        }

        info!("Successfully built template: {}", name);
        Ok(())
    }

    fn get_template(&self, platform: &str, name: &str) -> anyhow::Result<&Template> {
        match platform {
            "windows" => self
                .template_config
                .windows
                .get(name)
                .context("Windows template not found"),
            "linux" => self
                .template_config
                .linux
                .get(name)
                .context("Linux template not found"),
            _ => anyhow::bail!("Unsupported platform: {}", platform),
        }
    }

    async fn run_ansible_provisioning(&self, ansible_config: &AnsibleConfig) -> anyhow::Result<()> {
        let mut cmd = Command::new("ansible-playbook");
        cmd.current_dir(&self.working_dir)
            .arg(&ansible_config.playbook);

        for (key, value) in &ansible_config.vars {
            cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        let output = cmd
            .output()
            .await
            .context("Failed to execute ansible-playbook")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Ansible provisioning failed: {}", error);
        }

        Ok(())
    }

    pub async fn validate(&self, platform: &str, name: &str) -> anyhow::Result<()> {
        let template = self.get_template(platform, name)?;

        let output = Command::new("packer")
            .current_dir(&self.working_dir)
            .arg("validate")
            .arg(&template.packer.template)
            .output()
            .await
            .context("Failed to validate template")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Template validation failed: {}", error);
        }

        Ok(())
    }

    pub async fn list_templates(&self) -> Vec<(&str, &str, &Template)> {
        let mut templates = Vec::new();

        for (name, template) in &self.template_config.windows {
            templates.push(("windows", name.as_str(), template));
        }

        for (name, template) in &self.template_config.linux {
            templates.push(("linux", name.as_str(), template));
        }

        templates
    }
}
