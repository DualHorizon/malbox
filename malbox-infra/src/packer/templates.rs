use crate::error::{Error, Result};
use dialoguer::theme::ColorfulTheme;
use hcl::{Block, Body};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
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
    pub var_type: VarType,
    pub default: Option<String>,
    pub description: Option<String>,
    pub required: bool,
    pub enum_values: Option<Vec<String>>,
    pub sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VarType {
    String,
    Number,
    Bool,
    List,
    Map,
}

impl fmt::Display for VarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarType::String => write!(f, "string"),
            VarType::Number => write!(f, "number"),
            VarType::Bool => write!(f, "boolean"),
            VarType::List => write!(f, "list"),
            VarType::Map => write!(f, "map"),
        }
    }
}

impl From<&str> for VarType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "string" => VarType::String,
            "number" => VarType::Number,
            "bool" => VarType::Bool,
            "list" => VarType::List,
            "map" => VarType::Map,
            _ => VarType::String,
        }
    }
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
            var_type: VarType::String,
            default: None,
            description: None,
            required: true,
            enum_values: None,
            sensitive: false,
        };

        for attr in block.body().attributes() {
            match attr.key() {
                "type" => var.var_type = attr.expr().to_string().as_str().into(),
                "default" => {
                    var.default = Some(attr.expr().to_string());
                    var.required = false;
                }
                "description" => var.description = Some(attr.expr().to_string()),
                "sensitive" => var.sensitive = attr.expr().to_string().parse().unwrap_or(false),
                "validation" => {
                    if let Some(enum_values) = self.parse_enum_validation(attr) {
                        var.enum_values = Some(enum_values);
                    }
                }
                _ => {}
            }
        }

        Ok(Some((var_name, var)))
    }

    fn parse_enum_validation(&self, attr: &hcl::Attribute) -> Option<Vec<String>> {
        // Parse validation rules that define enum values
        // Example: validation { condition = contains(["a", "b", "c"], var.value) }
        None // Implementation depends on your HCL validation syntax
    }

    pub fn display_template_info(&self, template: &Template) {
        println!("\nTemplate Information:");
        println!("==============================");

        println!("\nRequired Variables:");
        for (name, var) in &template.variables {
            if var.required {
                self.display_variable(name, var);
            }
        }

        println!("\nOptional Variables:");
        for (name, var) in &template.variables {
            if !var.required {
                self.display_variable(name, var);
            }
        }
    }

    fn display_variable(&self, name: &str, var: &Variable) {
        println!("{0} - type: {1:?}", name, var.var_type);
        if let Some(desc) = &var.description {
            println!("  Description: {}", desc);
        }
        if let Some(default) = &var.default {
            println!("  Default: {}", default);
        }
        if var.sensitive {
            println!("  (Sensitive value)");
        }
        if let Some(enum_values) = &var.enum_values {
            println!("  Allowed values: {:?}", enum_values);
        }
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

    pub async fn prompt_for_variables(
        &self,
        template: &Template,
        provided: &mut HashMap<String, String>,
    ) -> Result<()> {
        let missing = self.get_missing_variables(template, provided)?;

        for var_name in missing {
            if let Some(var) = template.variables.get(&var_name) {
                let prompt = format!(
                    "Enter value for '{}' ({}){}",
                    var_name,
                    var.var_type,
                    if var.sensitive { " (sensitive)" } else { "" }
                );

                if let Some(enum_values) = &var.enum_values {
                    println!("Allowed values: {:?}", enum_values);
                }

                let value: String = if var.sensitive {
                    dialoguer::Password::with_theme(&ColorfulTheme::default())
                        .with_prompt(&prompt)
                        .interact()?
                } else {
                    dialoguer::Input::with_theme(&ColorfulTheme::default())
                        .with_prompt(&prompt)
                        .default(var.default.clone().unwrap_or_default())
                        .interact()?
                };

                provided.insert(var_name, value);
            }
        }

        Ok(())
    }
}
