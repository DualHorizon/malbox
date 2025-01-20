use clap::Parser;
use dialoguer::Confirm;
use malbox_config::Config;
use malbox_config::Config;

#[derive(Parser)]
pub struct DestroyArgs {
    #[arg(short, long)]
    pub environment: String,
    #[arg(short, long)]
    pub auto_approve: bool,
    #[arg(short, long)]
    pub target: Option<String>,
}

impl Command for DestroyArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        if !self.auto_approve {
            if !Confirm::new()
                .with_prompt("Do you really want to destroy this infrastructure?")
                .with_help_message("This action cannot be undone")
                .interact()?
            {
                return Ok(());
            }
        }

        Progress::new()
            .run(
                &format!(
                    "Destroying infrastructure in environment: {}",
                    self.environment
                ),
                async { Ok(()) },
            )
            .await
    }
}
