use clap::Subcommand;
pub mod builder;
pub mod daemon;

use builder::BuilderCommands;
use daemon::DaemonCommands;

#[derive(Subcommand, Debug)]
pub enum Modules {
    Builder {
        #[command(subcommand)]
        command: BuilderCommands,
    },
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
}
