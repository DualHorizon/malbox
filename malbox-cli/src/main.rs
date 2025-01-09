use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    modules: Modules,
}

#[derive(Subcommand, Debug)]
enum Modules {
    #[command(subcommand)]
    Builder,
    Daemon,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.modules {
        Modules::Builder { .. } => {
            println!("yup this is the builder");
            Ok(())
        }
        Modules::Daemon { .. } => malbox_daemon::run().await,
    }
}
