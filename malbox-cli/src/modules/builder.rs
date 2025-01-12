use clap::{Args, Subcommand};
use malbox_builder::Builder;
use malbox_config::{templates::TemplateConfig, Config};
use std::collections::HashMap;

#[derive(Subcommand, Debug)]
pub enum BuilderCommands {
    Build(BuildArgs),
    Validate(ValidateArgs),
    Init(InitArgs),
    List,
}

#[derive(Args, Debug)]
pub struct BuildArgs {
    #[arg(short, long)]
    platform: String,
    #[arg(short, long)]
    name: String,
    #[arg(short, long)]
    force: bool,
    #[arg(long)]
    hypervisor: String,
    #[arg(short, long)]
    working_dir: Option<String>,
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    #[arg(short, long)]
    platform: String,
    #[arg(short, long)]
    name: String,
    #[arg(short, long)]
    working_dir: Option<String>,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(short, long)]
    working_dir: Option<String>,
}

pub async fn handle_command(command: &BuilderCommands, config: &Config) -> anyhow::Result<()> {
    match command {
        BuilderCommands::Build(args) => handle_build(args, config).await,
        BuilderCommands::Validate(args) => handle_validate(args, config).await,
        BuilderCommands::List => handle_list(config).await,
        BuilderCommands::Init(args) => handle_init(args).await,
    }
}

async fn handle_build(args: &BuildArgs, config: &Config) -> anyhow::Result<()> {
    let builder = if let Some(working_dir) = &args.working_dir {
        Builder::with_working_dir(config.templates.clone(), working_dir)
    } else {
        Builder::new(config.templates.clone())
    };
    builder
        .build(&args.platform, &args.name, &args.hypervisor, args.force)
        .await
}

async fn handle_validate(args: &ValidateArgs, config: &Config) -> anyhow::Result<()> {
    let builder = if let Some(working_dir) = &args.working_dir {
        Builder::with_working_dir(config.templates.clone(), working_dir)
    } else {
        Builder::new(config.templates.clone())
    };
    builder.validate(&args.platform, &args.name).await
}

async fn handle_list(config: &Config) -> anyhow::Result<()> {
    let builder = Builder::new(config.templates.clone());
    println!("Available templates:");
    for (platform, name, template) in builder.list_templates().await {
        println!("- {} ({}):", name, platform);
        println!("  Description: {}", template.description);
        println!("  Packer template: {}", template.packer.template);
        if let Some(ansible) = &template.ansible {
            println!("  Ansible playbook: {}", ansible.playbook);
        }
        println!();
    }
    Ok(())
}

async fn handle_init(args: &InitArgs) -> anyhow::Result<()> {
    let empty_config = TemplateConfig {
        windows: HashMap::new(),
        linux: HashMap::new(),
    };

    let builder = if let Some(working_dir) = &args.working_dir {
        Builder::with_working_dir(empty_config, working_dir)
    } else {
        Builder::new(empty_config)
    };

    builder.init().await
}
