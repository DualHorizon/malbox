use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
    Start,
}

pub async fn handle_command(command: &DaemonCommands) -> anyhow::Result<()> {
    match command {
        DaemonCommands::Start => malbox_daemon::run().await,
    }
}
