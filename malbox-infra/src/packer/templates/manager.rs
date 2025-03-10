use super::{vars::VarType, Provisioner, Source, Template, TemplateDependencies, Variable};
use crate::error::{Error, Result};
use hcl::{Block, Body};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct TemplateManager {}

impl TemplateManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn load(&self, path: PathBuf) -> Result<Template> {
        let content = fs::read_to_string(&path).await?;
        let mut parsed = self.parse_template(&content)?;

        let display_name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        parsed.name = display_name;
        parsed.path = Some(path);

        Ok(parsed)
    }

    pub async fn find_templates(&self, base_dir: &Path) -> Result<Vec<Template>> {
        let mut results = Vec::new();

        if !base_dir.exists() {
            return Ok(results);
        }

        self.find_templates_in_dir(base_dir, &mut results).await?;

        Ok(results)
    }

    async fn find_templates_in_dir(&self, dir: &Path, results: &mut Vec<Template>) -> Result<()> {
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                Box::pin(self.find_templates_in_dir(&path, results)).await?;
            } else if let Some(ext) = path.extension() {
                if ext == "hcl" {
                    if let Ok(content) = fs::read_to_string(&path).await {
                        if content.contains("source")
                            && (content.contains("build {") || content.contains("build{"))
                        {
                            if let Ok(mut template) = self.parse_template(&content) {
                                template.name = path
                                    .file_stem()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                template.path = Some(path);

                                if let Ok(body) = hcl::from_str::<Body>(&content) {
                                    template.description =
                                        self.extract_description_from_body(&body);
                                }

                                results.push(template);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn extract_description_from_body(&self, body: &Body) -> Option<String> {
        for structure in body.iter() {
            if let hcl::Structure::Block(block) = structure {
                if block.identifier() == "variable" {
                    if let Some(var_name) = block.labels().first() {
                        if var_name.as_str() == "description" {
                            for attr in block.body().attributes() {
                                if attr.key() == "default" {
                                    return Some(attr.expr().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        None
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

    pub fn get_missing_variables(
        &self,
        template: &Template,
        provided: &HashMap<String, String>,
    ) -> Result<Vec<String>> {
        template.get_missing_variables(provided)
    }

    pub fn get_variable_info(&self, template: &Template) -> Vec<(String, bool, Option<String>)> {
        template
            .variables
            .iter()
            .map(|(name, var)| (name.clone(), var.required, var.description.clone()))
            .collect()
    }

    fn parse_template(&self, content: &str) -> Result<Template> {
        let body: Body = hcl::from_str(content)?;
        let mut variables = HashMap::new();
        let mut sources = Vec::new();
        let mut provisioners = Vec::new();
        let mut dependencies = TemplateDependencies::default();
        let mut description = None;

        for structure in body.iter() {
            match structure {
                hcl::Structure::Block(block) => match block.identifier().as_ref() {
                    "variable" => {
                        if let Some(var) = self.parse_variable(block)? {
                            if var.0 == "description" {
                                if let Some(default) = &var.1.default {
                                    description = Some(default.clone());
                                }
                            }
                            variables.insert(var.0, var.1);
                        }
                    }
                    "source" => {
                        if let Some(source) = self.parse_source(block)? {
                            self.extract_source_dependencies(block, &mut dependencies)?;
                            sources.push(source);
                        }
                    }
                    "build" => {
                        self.extract_build_dependencies(block, &mut dependencies)?;
                    }
                    "provisioner" => {
                        if let Some(provisioner) = self.parse_provisioner(block)? {
                            self.extract_provisioner_dependencies(block, &mut dependencies)?;
                            provisioners.push(provisioner);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(Template::builder()
            .name(String::new())
            .content(content.to_string())
            .variables(variables)
            .sources(sources)
            .provisioners(provisioners)
            .dependencies(dependencies)
            .maybe_description(description)
            .build())
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

    fn extract_source_dependencies(
        &self,
        block: &Block,
        deps: &mut TemplateDependencies,
    ) -> Result<()> {
        for attr in block.body().attributes() {
            match attr.key() {
                "http_directory" => {
                    if let Some(dir) = self.extract_string_value(attr.expr()) {
                        deps.http_directories.insert(dir);
                    }
                }
                "floppy_files" => {
                    self.extract_file_list(attr.expr(), &mut deps.floppy_files)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn extract_build_dependencies(
        &self,
        block: &Block,
        deps: &mut TemplateDependencies,
    ) -> Result<()> {
        for structure in block.body().iter() {
            if let hcl::Structure::Block(inner_block) = structure {
                if inner_block.identifier() == "provisioner" {
                    self.extract_provisioner_dependencies(inner_block, deps)?;
                }
            }
        }
        Ok(())
    }

    fn extract_provisioner_dependencies(
        &self,
        block: &Block,
        deps: &mut TemplateDependencies,
    ) -> Result<()> {
        if let Some(provisioner_type) = block.labels().first() {
            match provisioner_type.as_str() {
                "shell" | "powershell" => {
                    for attr in block.body().attributes() {
                        match attr.key() {
                            "scripts" => {
                                self.extract_file_list(attr.expr(), &mut deps.script_files)?;
                            }
                            "script" => {
                                if let Some(script) = self.extract_string_value(attr.expr()) {
                                    self.extract_filename(&script, &mut deps.script_files);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "ansible" => {
                    for attr in block.body().attributes() {
                        if attr.key() == "playbook_file" {
                            if let Some(playbook) = self.extract_string_value(attr.expr()) {
                                self.extract_filename(&playbook, &mut deps.provisioner_files);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn extract_string_value(&self, expr: &hcl::Expression) -> Option<String> {
        match expr {
            hcl::Expression::String(s) => Some(s.to_string()),
            _ => {
                let s = expr.to_string();
                if s.starts_with('"') && s.ends_with('"') {
                    Some(s.trim_matches('"').to_string())
                } else {
                    None
                }
            }
        }
    }

    fn extract_filename(&self, path: &str, set: &mut HashSet<String>) {
        if let Some(filename) = Path::new(path).file_name() {
            if let Some(name) = filename.to_str() {
                set.insert(name.to_string());
            }
        }
    }

    fn extract_file_list(&self, expr: &hcl::Expression, files: &mut HashSet<String>) -> Result<()> {
        match expr {
            hcl::Expression::Array(items) => {
                for item in items {
                    if let Some(path) = self.extract_string_value(item) {
                        self.extract_filename(&path, files);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
