use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

mod manager;
mod vars;

pub use manager::{TemplateInfo, TemplateManager};
pub use vars::Variable;

// IMPORTANT - We only support HCL syntax for packer templates, no JSON
// Since this is the new recommended way to describe templates, we prefer
// to leave JSON behind
// ---------------------
// Currently the structure for this is not the best, we should try to separate by logical concerns
// ex. the parsing logic would be in parser.rs, manager should probably be at top level, etc..

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub path: Option<PathBuf>,
    pub variables: HashMap<String, Variable>,
    pub sources: Vec<Source>,
    pub provisioners: Vec<Provisioner>,
    pub content: String,
}

impl Template {
    pub fn get_missing_variables(&self, provided: &HashMap<String, String>) -> Result<Vec<String>> {
        let mut missing = Vec::new();
        for (name, var) in &self.variables {
            if var.required && !provided.contains_key(name) {
                missing.push(name.clone());
            }
        }
        Ok(missing)
    }

    pub fn validate_all_variables(&self, variables: &HashMap<String, String>) -> Result<()> {
        let mut errors = Vec::new();

        for (name, var) in &self.variables {
            if let Some(value) = variables.get(name) {
                if let Err(e) = var.validate_and_format(value) {
                    errors.push(format!("{}: {}", name, e));
                }
            } else if var.required {
                errors.push(format!("{}: Missing required variable", name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Variable(errors.join("\n")))
        }
    }
}
