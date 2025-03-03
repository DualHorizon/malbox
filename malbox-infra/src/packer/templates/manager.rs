use super::{vars::VarType, Provisioner, Source, Template, Variable};
use crate::error::{Error, Result};
use hcl::{Block, Body};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct TemplateManager {}

impl TemplateManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn load(&self, path: PathBuf) -> Result<Template> {
        let content = fs::read_to_string(&path).await?;
        let parsed = self.parse_template(&content)?;

        // Add the path and file name information to the template
        let display_name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Ok(Template {
            name: display_name,
            path: Some(path),
            content,
            ..parsed
        })
    }

    pub async fn find_templates(&self, base_dir: &Path) -> Result<Vec<TemplateInfo>> {
        let mut results = Vec::new();

        if !base_dir.exists() {
            return Ok(results);
        }

        self.find_templates_in_dir(base_dir, &mut results).await?;

        Ok(results)
    }

    async fn find_templates_in_dir(
        &self,
        dir: &Path,
        results: &mut Vec<TemplateInfo>,
    ) -> Result<()> {
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                // Use Box::pin to handle recursion in async function
                Box::pin(self.find_templates_in_dir(&path, results)).await?;
            } else if let Some(ext) = path.extension() {
                if ext == "hcl" {
                    // Check if it's a packer template
                    if let Ok(content) = fs::read_to_string(&path).await {
                        if content.contains("source")
                            && (content.contains("build {") || content.contains("build{"))
                        {
                            // Quick parse to get basic info
                            if let Ok(info) = self.extract_template_info(&path, &content).await {
                                results.push(info);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn extract_template_info(&self, path: &Path, content: &str) -> Result<TemplateInfo> {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let description = if let Ok(body) = hcl::from_str::<Body>(content) {
            // Try to find a description in packer variables
            for structure in body.iter() {
                if let hcl::Structure::Block(block) = structure {
                    if block.identifier() == "variable" {
                        if let Some(var_name) = block.labels().first() {
                            if var_name.as_str() == "description" {
                                for attr in block.body().attributes() {
                                    if attr.key() == "default" {
                                        return Ok(TemplateInfo {
                                            name: name.clone(),
                                            path: path.to_path_buf(),
                                            description: Some(attr.expr().to_string()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // If we get here, no description was found
            None
        } else {
            None
        };

        Ok(TemplateInfo {
            name,
            path: path.to_path_buf(),
            description,
        })
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
            name: String::new(), // Will be populated by the caller
            path: None,          // Will be populated by the caller
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
        // Try to extract enum values from validation rules
        // This is a more sophisticated version that tries to handle different validation patterns

        let expr_str = attr.expr().to_string();

        // Simple pattern: contains(["value1", "value2"], var.xyz)
        if expr_str.contains("contains(") && expr_str.contains("[") && expr_str.contains("]") {
            let start = expr_str.find('[').unwrap_or(0);
            let end = expr_str.find(']').unwrap_or(expr_str.len());

            if start < end && start > 0 {
                let values_str = &expr_str[start + 1..end];
                return Some(
                    values_str
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                        .collect(),
                );
            }
        }

        None
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

/// Basic information about a template without fully parsing it
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
}
