use clap::Parser;
use color_eyre::Result;

mod commands;
mod error;
mod types;
mod utils;

use commands::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let config = malbox_config::load_config().await?;
    cli.execute(&config)
        .await
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))
}
