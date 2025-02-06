use crate::{
    commands::Command,
    error::Result,
    utils::{interaction::templates::TemplatePrompt, progress::Progress},
};
use clap::Parser;
use malbox_config::Config;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
pub struct DownloadArgs {
    #[arg(short, long)]
    pub name: String,
    #[arg(short, long)]
    pub url: String,
}

impl Command for DownloadArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        malbox_downloader::download_files("https://delivery.activated.win/f6f032ff-a234-46f0-8c5c-4dfc81173b29/en-us_windows_10_consumer_editions_version_22h2_updated_nov_2024_x64_dvd_3eeacab9.iso?t=641515f4-e797-4392-8cd5-244110e2a0dc&P1=1738868818&P2=601&P3=2&P4=8CPr9wp5SyNCn9JWiXXdc%2Br9i%2BAwmEQp50%2FZC4ixNFQ%3D", "/home/shard/Downloads/testfiledl.iso").await?;

        Ok(())
    }
}
