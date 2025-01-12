use clap::Parser;
mod modules;
use modules::{builder, daemon, Modules};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    module: Modules,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = malbox_config::load_config().await?;

    match &cli.module {
        Modules::Builder { command } => builder::handle_command(command, config).await,
        Modules::Daemon { command } => daemon::handle_command(command).await,
    }
}
