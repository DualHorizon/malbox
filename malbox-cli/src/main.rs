use clap::Parser;
use color_eyre::Result;
use malbox_tracing::init_tracing;

mod commands;
mod error;
mod types;
mod utils;

use commands::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = malbox_config::load_config().await?;

    init_tracing(&config.general.log_level.to_string());

    let cli = Cli::parse();

    cli.execute(&config)
        .await
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))
}
