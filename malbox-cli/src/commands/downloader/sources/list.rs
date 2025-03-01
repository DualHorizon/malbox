use crate::{commands::Command, error::Result, types::OutputFormat};
use byte_unit::{Byte, UnitType};
use clap::Parser;
use console::{style, Term};
use malbox_config::Config;
use malbox_downloader::{
    SourceEdition, SourceFamily, SourceRegistry, SourceRelease, SourceVariant,
};

#[derive(Parser)]
pub struct ListSourcesArgs {
    #[arg(short = 'f', long)]
    pub family: Option<String>,
    #[arg(short = 'e', long)]
    pub edition: Option<String>,
    #[arg(short = 'v', long)]
    pub version: Option<String>,
    #[arg(short, long)]
    pub detailed: bool,
    #[arg(value_enum, long, default_value = "text")]
    pub format: OutputFormat,
}

impl Command for ListSourcesArgs {
    async fn execute(self, config: &Config) -> Result<()> {
        let registry_path = config.paths.download_dir.join("download_registry.json");
        let registry = SourceRegistry::load(registry_path).await?;
        let term = Term::stdout();

        match self.format {
            OutputFormat::Json => {
                if let Some(family_id) = &self.family {
                    if let Some(edition_id) = &self.edition {
                        if let Some(version) = &self.version {
                            let variants =
                                registry.list_variants(family_id, edition_id, version)?;
                            println!("{}", serde_json::to_string_pretty(&variants)?);
                        } else {
                            let releases = registry.list_releases(family_id, edition_id)?;
                            println!("{}", serde_json::to_string_pretty(&releases)?);
                        }
                    } else {
                        let editions = registry.list_editions(family_id)?;
                        println!("{}", serde_json::to_string_pretty(&editions)?);
                    }
                } else {
                    let families = registry.list_families();
                    println!("{}", serde_json::to_string_pretty(&families)?);
                }
            }
            OutputFormat::Yaml => {
                if let Some(family_id) = &self.family {
                    if let Some(edition_id) = &self.edition {
                        if let Some(version) = &self.version {
                            let variants =
                                registry.list_variants(family_id, edition_id, version)?;
                            println!("{}", serde_yaml::to_string(&variants)?);
                        } else {
                            let releases = registry.list_releases(family_id, edition_id)?;
                            println!("{}", serde_yaml::to_string(&releases)?);
                        }
                    } else {
                        let editions = registry.list_editions(family_id)?;
                        println!("{}", serde_yaml::to_string(&editions)?);
                    }
                } else {
                    let families = registry.list_families();
                    println!("{}", serde_yaml::to_string(&families)?);
                }
            }
            OutputFormat::Text => {
                term.write_line(&format!(
                    "\n{}",
                    style("Available Sources").bold().underlined()
                ))?;

                if let Some(family_id) = &self.family {
                    let family_exists = registry.list_families().iter().any(|f| f.id == *family_id);

                    if !family_exists {
                        term.write_line(&format!(
                            "\n{} {}",
                            style("No sources found for family").red(),
                            style(family_id).cyan()
                        ))?;
                        return Ok(());
                    }

                    if let Some(edition_id) = &self.edition {
                        if let Some(version) = &self.version {
                            match registry.list_variants(family_id, edition_id, version) {
                                Ok(variants) => {
                                    term.write_line(&format!(
                                        "\n{} {}/{}/{}:",
                                        style("Variants for").bold(),
                                        style(family_id).cyan(),
                                        style(edition_id).cyan(),
                                        style(version).cyan(),
                                    ))?;

                                    for variant in variants {
                                        print_variant(&term, variant, self.detailed)?;
                                    }
                                }
                                Err(_) => {
                                    term.write_line(&format!(
                                        "\n{} {}/{}/{}",
                                        style("No variants found for").red(),
                                        style(family_id).cyan(),
                                        style(edition_id).cyan(),
                                        style(version).cyan(),
                                    ))?;
                                }
                            }
                        } else {
                            match registry.list_releases(family_id, edition_id) {
                                Ok(releases) => {
                                    term.write_line(&format!(
                                        "\n{} {}/{}:",
                                        style("Releases for").bold(),
                                        style(family_id).cyan(),
                                        style(edition_id).cyan(),
                                    ))?;

                                    for release in releases {
                                        print_release(&term, release, self.detailed)?;
                                    }
                                }
                                Err(_) => {
                                    term.write_line(&format!(
                                        "\n{} {}/{}",
                                        style("No releases found for").red(),
                                        style(family_id).cyan(),
                                        style(edition_id).cyan(),
                                    ))?;
                                }
                            }
                        }
                    } else {
                        match registry.list_editions(family_id) {
                            Ok(editions) => {
                                term.write_line(&format!(
                                    "\n{} {}:",
                                    style("Editions for").bold(),
                                    style(family_id).cyan(),
                                ))?;

                                for edition in editions {
                                    print_edition(&term, edition, self.detailed)?;
                                }
                            }
                            Err(_) => {
                                term.write_line(&format!(
                                    "\n{} {}",
                                    style("No editions found for").red(),
                                    style(family_id).cyan(),
                                ))?;
                            }
                        }
                    }
                } else {
                    let families = registry.list_families();
                    tracing::debug!("Families: {:#?}", families);

                    if families.is_empty() {
                        term.write_line("\nNo sources found in registry.")?;
                        return Ok(());
                    }

                    for family in families {
                        print_family(&term, family, self.detailed)?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn print_family(term: &Term, family: &SourceFamily, detailed: bool) -> std::io::Result<()> {
    term.write_line(&format!(
        "\n{} {}:",
        style("Family").bold(),
        style(&family.name).cyan().bold()
    ))?;

    if detailed {
        term.write_line(&format!("  {}: {}", style("ID").dim(), family.id))?;
        term.write_line(&format!(
            "  {}: {}",
            style("Description").dim(),
            family.description
        ))?;
        term.write_line(&format!(
            "  {}: {:?}",
            style("Platform").dim(),
            family.platform
        ))?;

        if !family.tags.is_empty() {
            term.write_line(&format!(
                "  {}: {}",
                style("Tags").dim(),
                family
                    .tags
                    .iter()
                    .map(|t| style(t).magenta().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))?;
        }

        term.write_line(&format!(
            "  {}: {}",
            style("Editions").dim(),
            family.editions.len()
        ))?;

        for edition in &family.editions {
            print_edition(term, edition, false)?;
        }
    } else {
        for edition in &family.editions {
            term.write_line(&format!(
                "  • {} ({} releases)",
                style(&edition.name).bold(),
                edition.releases.len()
            ))?;
        }
    }

    Ok(())
}

fn print_edition(term: &Term, edition: &SourceEdition, detailed: bool) -> std::io::Result<()> {
    if detailed {
        term.write_line(&format!(
            "\n  {} {} ({} releases)",
            style("▶").cyan(),
            style(&edition.name).bold(),
            edition.releases.len()
        ))?;

        term.write_line(&format!("    {}: {}", style("ID").dim(), edition.id))?;
        term.write_line(&format!(
            "    {}: {}",
            style("Description").dim(),
            edition.description
        ))?;

        for release in &edition.releases {
            print_release(term, release, false)?;
        }
    } else {
        term.write_line(&format!(
            "  • {} ({} releases)",
            style(&edition.name).bold(),
            edition.releases.len()
        ))?;
    }

    Ok(())
}

fn print_release(term: &Term, release: &SourceRelease, detailed: bool) -> std::io::Result<()> {
    if detailed {
        term.write_line(&format!(
            "\n    {} {} ({} variants)",
            style("▶").cyan(),
            style(&release.version).bold(),
            release.variants.len()
        ))?;

        term.write_line(&format!(
            "      {}: {}",
            style("Description").dim(),
            release.description
        ))?;

        if let Some(release_date) = &release.release_date {
            term.write_line(&format!(
                "      {}: {}",
                style("Released").dim(),
                release_date.date()
            ))?;
        }

        if let Some(eol_date) = &release.eol_date {
            term.write_line(&format!(
                "      {}: {}",
                style("EOL").dim(),
                eol_date.date()
            ))?;
        }

        if let Some(notes) = &release.release_notes {
            term.write_line(&format!(
                "      {}: {}",
                style("Release Notes").dim(),
                notes
            ))?;
        }

        for variant in &release.variants {
            print_variant(term, variant, false)?;
        }
    } else {
        term.write_line(&format!(
            "    • {} ({} variants)",
            style(&release.version).bold(),
            release.variants.len()
        ))?;
    }

    Ok(())
}

fn print_variant(term: &Term, variant: &SourceVariant, detailed: bool) -> std::io::Result<()> {
    if detailed {
        term.write_line(&format!(
            "\n      {} {} [{}] [{}]",
            style("▶").cyan(),
            style(&variant.id).bold(),
            style(format!("{:?}", variant.architecture)).yellow(),
            style(format!("{:?}", variant.source_type)).blue()
        ))?;

        term.write_line(&format!(
            "        {}: {}",
            style("Description").dim(),
            variant.description
        ))?;

        term.write_line(&format!(
            "        {}: {}",
            style("URL").dim(),
            style(&variant.url).blue().underlined()
        ))?;

        if let Some(ref doc_url) = variant.documentation_url {
            term.write_line(&format!(
                "        {}: {}",
                style("Documentation").dim(),
                style(doc_url).blue().underlined()
            ))?;
        }

        if let Some(size) = variant.size {
            let byte = Byte::from_u128(size as u128).unwrap_or_default();
            let adjusted_byte = byte.get_appropriate_unit(UnitType::Decimal);
            term.write_line(&format!(
                "        {}: {}",
                style("Size").dim(),
                adjusted_byte
            ))?;
        }

        if let Some(checksum) = &variant.checksum {
            term.write_line(&format!(
                "        {}: {} ({})",
                style("Checksum").dim(),
                checksum,
                style(variant.checksum_type.as_deref().unwrap_or("unknown")).yellow()
            ))?;
        }

        // if let Some(ref reqs) = variant.minimum_requirements {
        //     term.write_line(&format!("        {}", style("System Requirements").dim()))?;
        //     term.write_line(&format!(
        //         "          CPU Cores: {}, Memory: {} MB, Disk: {} GB",
        //         reqs.cpu_cores, reqs.memory_mb, reqs.disk_gb
        //     ))?;
        // }

        term.write_line(&format!(
            "        {}: {:?}",
            style("Status").dim(),
            variant.metadata.processing_status
        ))?;

        if let Some(ref parent) = variant.metadata.parent_source {
            term.write_line(&format!(
                "        {}: {}",
                style("Derived from").dim(),
                parent
            ))?;
        }

        term.write_line(&format!(
            "        {}: Added: {}, Downloads: {}",
            style("Stats").dim(),
            variant.metadata.added_date.date(),
            variant.metadata.downloads_count
        ))?;

        if let Some(verified) = variant.metadata.last_verified {
            term.write_line(&format!("            Last verified: {}", verified.date()))?;
        }

        if let Some(downloaded) = variant.metadata.last_downloaded {
            term.write_line(&format!(
                "            Last downloaded: {}",
                downloaded.date()
            ))?;
        }

        if let Some(local_path) = &variant.metadata.local_path {
            term.write_line(&format!(
                "        {}: {}",
                style("Local path").dim(),
                local_path
            ))?;
        }

        if !variant.mirrors.is_empty() {
            term.write_line(&format!("        {}", style("Mirrors").dim()))?;
            for mirror in &variant.mirrors {
                term.write_line(&format!(
                    "          {} {}",
                    style("•").cyan(),
                    style(mirror).blue().underlined()
                ))?;
            }
        }
    } else {
        term.write_line(&format!(
            "      • {} [{:?}] [{:?}]",
            style(&variant.id).bold(),
            variant.architecture,
            variant.source_type
        ))?;
    }

    Ok(())
}
