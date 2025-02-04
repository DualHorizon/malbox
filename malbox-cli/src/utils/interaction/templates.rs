use crate::error::Result;
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use malbox_infra::packer::templates::{Template, Variable};
use std::collections::HashMap;

pub struct TemplatePrompt {
    theme: ColorfulTheme,
}

impl Default for TemplatePrompt {
    fn default() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }
}

impl TemplatePrompt {
    pub async fn prompt_variables(
        &self,
        template: &Template,
        provided: &mut HashMap<String, String>,
    ) -> Result<()> {
        let missing = template.get_missing_variables(provided)?;

        for var_name in missing {
            if let Some(var) = template.variables.get(&var_name) {
                self.prompt_variable(&var_name, var, provided)?;
            }
        }

        Ok(())
    }

    fn prompt_variable(
        &self,
        name: &str,
        var: &Variable,
        values: &mut HashMap<String, String>,
    ) -> Result<()> {
        loop {
            let prompt = format!(
                "{} ({}){}",
                style(name).green().bold(),
                style(&var.var_type).blue(),
                if var.sensitive {
                    style(" (sensitive)").red().to_string()
                } else {
                    String::new()
                }
            );

            let value = if let Some(enum_values) = &var.enum_values {
                let selection = Select::with_theme(&self.theme)
                    .with_prompt(&prompt)
                    .items(enum_values)
                    .default(0)
                    .interact()?;
                enum_values[selection].clone()
            } else if var.sensitive {
                dialoguer::Password::with_theme(&self.theme)
                    .with_prompt(&prompt)
                    .interact()?
            } else {
                Input::with_theme(&self.theme)
                    .with_prompt(&prompt)
                    .default(var.default.clone().unwrap_or_default())
                    .interact_text()?
            };

            match var.validate_and_format(&value) {
                Ok(formatted_value) => {
                    values.insert(name.to_string(), formatted_value);
                    break;
                }
                Err(e) => {
                    println!("{}", style(e).red());
                    println!("Please try again.");
                    continue;
                }
            }
        }

        Ok(())
    }

    pub fn display_template_info(&self, template: &Template) -> Result<()> {
        let term = Term::stdout();
        term.clear_screen()?;

        println!("{}", style("Template Information").cyan().bold());
        println!("{}", style("===================").cyan());

        // Sources
        if !template.sources.is_empty() {
            println!("{}", style("Sources:").yellow().bold());
            for source in &template.sources {
                println!("  {} ({})", style(&source.name).green(), source.source_type);
            }
        }

        // Variables
        let (required, optional): (Vec<_>, Vec<_>) =
            template.variables.iter().partition(|(_, var)| var.required);

        if !required.is_empty() {
            println!("{}", style("Required Variables:").yellow().bold());
            for (name, var) in required {
                self.display_variable(name, var)?;
            }
        }

        if !optional.is_empty() {
            println!("{}", style("Optional Variables:").yellow().bold());
            for (name, var) in optional {
                self.display_variable(name, var)?;
            }
        }

        println!();
        Ok(())
    }

    fn display_variable(&self, name: &str, var: &Variable) -> Result<()> {
        println!("{}", style(name).green().bold());
        println!("    Type: {}", style(&var.var_type).blue());

        if let Some(desc) = &var.description {
            println!("    Description: {}", desc);
        }

        if let Some(default) = &var.default {
            println!("    Default: {}", style(default).yellow());
        }

        if var.sensitive {
            println!("    {}", style("(Sensitive value)").red());
        }

        if let Some(enum_values) = &var.enum_values {
            println!(
                "    Allowed values: {}",
                style(enum_values.join(", ")).cyan()
            );
        }

        Ok(())
    }
}
