use crate::{commands::Command, error::Result, types::OutputFormat};
use byte_unit::{Byte, UnitType};
use clap::Parser;
use console::{style, Term};
use malbox_config::Config;
use malbox_downloader::{DownloadRegistry, DownloadSource};

#[derive(Parser)]
pub struct ListSourcesArgs {
    #[arg(short, long)]
    pub category: Option<String>,
    #[arg(value_enum, short, long, default_value = "text")]
    pub format: OutputFormat,
    #[arg(short, long)]
    pub detailed: bool,
}

impl Command for ListSourcesArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let registry_path = config.paths.download_dir.join("download_registry.json");
        let registry = DownloadRegistry::load(registry_path).await?;
        let term = Term::stdout();

        match self.format {
            OutputFormat::Json => {
                let mut all_sources = registry.sources;
                if !registry.custom_sources.is_empty() {
                    all_sources.insert(
                        "custom".to_string(),
                        registry.custom_sources.into_values().collect(),
                    );
                }

                if let Some(category) = self.category {
                    if let Some(sources) = all_sources.get(&category) {
                        println!("{}", serde_json::to_string_pretty(sources)?);
                    }
                } else {
                    println!("{}", serde_json::to_string_pretty(&all_sources)?);
                }
            }
            OutputFormat::Yaml => {
                let mut all_sources = registry.sources;
                if !registry.custom_sources.is_empty() {
                    all_sources.insert(
                        "custom".to_string(),
                        registry.custom_sources.into_values().collect(),
                    );
                }

                if let Some(category) = self.category {
                    if let Some(sources) = all_sources.get(&category) {
                        println!("{}", serde_yaml::to_string(sources)?);
                    }
                } else {
                    println!("{}", serde_yaml::to_string(&all_sources)?);
                }
            }
            OutputFormat::Text => {
                term.write_line(&format!(
                    "\n{}",
                    style("Available Sources").bold().underlined()
                ))?;

                if let Some(category) = self.category {
                    if category == "custom" && !registry.custom_sources.is_empty() {
                        print_category(
                            &term,
                            "Custom",
                            &registry
                                .custom_sources
                                .values()
                                .cloned()
                                .collect::<Vec<_>>(),
                            self.detailed,
                        )?;
                    } else if let Some(sources) = registry.sources.get(&category) {
                        print_category(&term, &category, sources, self.detailed)?;
                    } else {
                        term.write_line(&format!(
                            "\n{} {}",
                            style("No sources found for category").red(),
                            style(category).cyan()
                        ))?;
                    }
                } else {
                    for (category, sources) in &registry.sources {
                        print_category(&term, category, sources, self.detailed)?;
                    }

                    if !registry.custom_sources.is_empty() {
                        print_category(
                            &term,
                            "Custom",
                            &registry
                                .custom_sources
                                .values()
                                .cloned()
                                .collect::<Vec<_>>(),
                            self.detailed,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn print_category(
    term: &Term,
    category: &str,
    sources: &[DownloadSource],
    detailed: bool,
) -> std::io::Result<()> {
    term.write_line(&format!(
        "\n{} {}:",
        style("Category").bold(),
        style(category).cyan().bold()
    ))?;

    for source in sources {
        print_source(term, source, detailed)?;
    }
    Ok(())
}

fn print_source(term: &Term, source: &DownloadSource, detailed: bool) -> std::io::Result<()> {
    if detailed {
        term.write_line(&format!(
            "\n  {} {} ({})",
            style("▶").cyan(),
            style(&source.name).bold(),
            style(&source.version).yellow()
        ))?;
        term.write_line(&format!(
            "    {} {}",
            style("Description:").dim(),
            source.description
        ))?;
        term.write_line(&format!(
            "    {} {}",
            style("URL:").dim(),
            style(&source.url).blue().underlined()
        ))?;

        if let Some(size) = source.size {
            let byte = Byte::from_u128(size as u128).unwrap_or_default();
            let adjusted_byte = byte.get_appropriate_unit(UnitType::Decimal);
            term.write_line(&format!("    {} {}", style("Size:").dim(), adjusted_byte))?;
        }

        if let Some(checksum) = &source.checksum {
            term.write_line(&format!(
                "    {} {} ({})",
                style("Checksum:").dim(),
                checksum,
                style(source.checksum_type.as_deref().unwrap_or("unknown")).yellow()
            ))?;
        }

        if !source.tags.is_empty() {
            term.write_line(&format!(
                "    {} {}",
                style("Tags:").dim(),
                source
                    .tags
                    .iter()
                    .map(|t| style(t).magenta().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))?;
        }

        if !source.mirrors.is_empty() {
            term.write_line(&format!("    {}", style("Mirrors:").dim()))?;
            for mirror in &source.mirrors {
                term.write_line(&format!(
                    "      {} {}",
                    style("•").cyan(),
                    style(mirror).blue().underlined()
                ))?;
            }
        }
    } else {
        term.write_line(&format!(
            "  {} {} ({})",
            style("•").cyan(),
            style(&source.name).bold(),
            style(&source.version).yellow()
        ))?;
    }
    Ok(())
}
