use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

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

impl VarType {
    pub fn validate_value(&self, value: &str) -> bool {
        match self {
            VarType::String => true,
            VarType::Number => value.parse::<f64>().is_ok(),
            VarType::Bool => matches!(value.to_lowercase().as_str(), "true" | "false" | "1" | "0"),
            VarType::List => value.starts_with('[') && value.ends_with(']'),
            VarType::Map => value.starts_with('{') && value.ends_with('}'),
        }
    }

    pub fn format_value(&self, value: &str) -> Result<String> {
        match self {
            VarType::String => Ok(value.to_string()),
            VarType::Number => value
                .parse::<f64>()
                .map(|n| n.to_string())
                .map_err(|_| Error::Variable(format!("Invalid number format: {}", value))),
            VarType::Bool => {
                let bool_value = match value.to_lowercase().as_str() {
                    "true" | "1" => "true",
                    "false" | "0" => "false",
                    _ => return Err(Error::Variable(format!("Invalid boolean value: {}", value))),
                };
                Ok(bool_value.to_string())
            }
            VarType::List => Ok(value.to_string()), // Could add JSON validation
            VarType::Map => Ok(value.to_string()),  // Could add JSON validation
        }
    }
}

impl Variable {
    pub fn validate_and_format(&self, value: &str) -> Result<String> {
        if !self.var_type.validate_value(value) {
            return Err(Error::Variable(format!(
                "Invalid value for type {}: {}",
                self.var_type, value
            )));
        }

        if let Some(enum_values) = &self.enum_values {
            if !enum_values.contains(&value.to_string()) {
                return Err(Error::Variable(format!(
                    "Value must be one of: {:?}",
                    enum_values
                )));
            }
        }

        self.var_type.format_value(value)
    }
}
