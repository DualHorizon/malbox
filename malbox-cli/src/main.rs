use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    module: Modules,
}

#[derive(Subcommand, Debug)]
enum Modules {
    Builder {
        #[command(subcommand)]
        command: BuilderCommands,
    },
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
}

#[derive(Subcommand, Debug)]
enum BuilderCommands {
    Pack,
    Init,
}

#[derive(Subcommand, Debug)]
enum DaemonCommands {
    Start,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.module {
        Modules::Builder { .. } => Ok(()),
        Modules::Daemon { command } => match command {
            DaemonCommands::Start { .. } => malbox_daemon::run().await,
        },
    }
}
