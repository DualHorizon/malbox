use crate::error::{Error, Result};
use hcl::{Block, Body};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub variables: HashMap<String, Variable>,
    pub sources: Vec<Source>,
    pub provisioners: Vec<Provisioner>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub var_type: String,
    pub default: Option<String>,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub source_type: String,
    pub name: String,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provisioner {
    pub provisioner_type: String,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TemplateConfig {
    pub template: String,
}

pub struct TemplateManager {}

impl TemplateManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn load(&self, path: PathBuf) -> Result<Template> {
        let content = tokio::fs::read_to_string(path).await?;
        let parsed = self.parse_template(&content)?;
        Ok(Template { content, ..parsed })
    }

    pub fn validate(&self, template: &Template, variables: &HashMap<String, String>) -> Result<()> {
        let missing: Vec<String> = template
            .variables
            .iter()
            .filter(|(name, var)| var.required && !variables.contains_key(*name))
            .map(|(name, _)| name.clone())
            .collect();

        if !missing.is_empty() {
            return Err(Error::Variable(format!(
                "Missing required variables: {}",
                missing.join(", ")
            )));
        }
        Ok(())
    }

    fn parse_template(&self, content: &str) -> Result<Template> {
        let body: Body = hcl::from_str(content)?;
        let mut variables = HashMap::new();
        let mut sources = Vec::new();
        let mut provisioners = Vec::new();

        for structure in body.iter() {
            match structure {
                hcl::Structure::Block(block) => match block.identifier().as_ref() {
                    "variable" => {
                        if let Some(var) = self.parse_variable(block)? {
                            variables.insert(var.0, var.1);
                        }
                    }
                    "source" => {
                        if let Some(source) = self.parse_source(block)? {
                            sources.push(source);
                        }
                    }
                    "provisioner" => {
                        if let Some(provisioner) = self.parse_provisioner(block)? {
                            provisioners.push(provisioner);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(Template {
            variables,
            sources,
            provisioners,
            content: content.to_string(),
        })
    }

    fn parse_variable(&self, block: &Block) -> Result<Option<(String, Variable)>> {
        let var_name = block
            .labels()
            .first()
            .ok_or_else(|| Error::Template("Variable missing name".to_string()))?
            .as_str()
            .to_string();

        let mut var = Variable {
            var_type: "string".to_string(),
            default: None,
            description: None,
            required: true,
        };

        for attr in block.body().attributes() {
            match attr.key() {
                "type" => var.var_type = attr.expr().to_string(),
                "default" => {
                    var.default = Some(attr.expr().to_string());
                    var.required = false;
                }
                "description" => var.description = Some(attr.expr().to_string()),
                _ => {}
            }
        }

        Ok(Some((var_name, var)))
    }

    fn parse_source(&self, block: &Block) -> Result<Option<Source>> {
        let labels: Vec<_> = block.labels().into();
        if labels.len() < 2 {
            return Err(Error::Template("Invalid source block".to_string()));
        }

        let mut config = HashMap::new();
        for attr in block.body().attributes() {
            config.insert(attr.key().to_string(), attr.expr().to_string());
        }

        Ok(Some(Source {
            source_type: labels[0].as_str().to_string(),
            name: labels[1].as_str().to_string(),
            config,
        }))
    }

    fn parse_provisioner(&self, block: &Block) -> Result<Option<Provisioner>> {
        let prov_type = block
            .labels()
            .first()
            .ok_or_else(|| Error::Template("Provisioner missing type".to_string()))?
            .as_str()
            .to_string();

        let mut config = HashMap::new();
        for attr in block.body().attributes() {
            config.insert(attr.key().to_string(), attr.expr().to_string());
        }

        Ok(Some(Provisioner {
            provisioner_type: prov_type,
            config,
        }))
    }

    pub fn get_missing_variables(
        &self,
        template: &Template,
        provided: &HashMap<String, String>,
    ) -> Result<Vec<String>> {
        let mut missing = Vec::new();

        for (name, var) in &template.variables {
            if var.required && !provided.contains_key(name) {
                missing.push(name.clone());
            }
        }

        Ok(missing)
    }

    pub fn get_variable_info(&self, template: &Template) -> Vec<(String, bool, Option<String>)> {
        template
            .variables
            .iter()
            .map(|(name, var)| (name.clone(), var.required, var.description.clone()))
            .collect()
    }
}

pub async fn show_template_info(template_path: PathBuf) -> Result<()> {
    let template_manager = TemplateManager::new();
    let template = template_manager.load(template_path.clone()).await?;

    println!("Template: {:#?}", template_path.to_str().unwrap());
    println!("\nRequired Variables:");
    for (name, required, desc) in template_manager.get_variable_info(&template) {
        if required {
            println!(
                "  {} - {}",
                name,
                desc.unwrap_or_else(|| "No description".to_string())
            );
        }
    }

    println!("\nOptional Variables:");
    for (name, required, desc) in template_manager.get_variable_info(&template) {
        if !required {
            println!(
                "  {} - {}",
                name,
                desc.unwrap_or_else(|| "No description".to_string())
            );
        }
    }

    Ok(())
}
