use crate::ConfigError;
use std::{collections::HashMap, path::PathBuf};

pub fn load_variables_from_file(path: &PathBuf) -> Result<HashMap<String, String>, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    toml::from_str(&content).map_err(|e| ConfigError::Parse {
        file: path.display().to_string(),
        error: e.to_string(),
    })
}

pub fn expand_env_vars(value: &str) -> String {
    let mut result = value.to_string();
    while let Some(start) = result.find("${") {
        if let Some(end) = result[start..].find('}') {
            let var_name = &result[start + 2..start + end];
            if let Ok(var_value) = std::env::var(var_name) {
                result.replace_range(start..start + end + 1, &var_value);
            }
        } else {
            break;
        }
    }
    result
}
