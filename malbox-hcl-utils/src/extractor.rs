use crate::{
    error::{HclError, Result},
    types::{FromHclExpression, HclValue},
};
use hcl::{Attribute, Block, Expression};
use std::collections::HashMap;

/// Extract a string value from an HCL expression.
pub fn extract_string(expr: &Expression) -> Option<String> {
    match expr {
        Expression::String(s) => Some(s.to_string()),
        _ => {
            let s = expr.to_string();
            if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                Some(s[1..s.len() - 1].to_string())
            } else {
                None
            }
        }
    }
}

/// Extract a number value from an HCL expression.
pub fn extract_number(expr: &Expression) -> Option<f64> {
    match expr {
        Expression::Number(n) => Some(*n),
        _ => expr.to_string().parse().ok(),
    }
}

/// Extract a boolean value from an HCL expression.
pub fn extract_bool(expr: &Expression) -> Option<bool> {
    match expr {
        Expression::Bool(b) => Some(*b),
        _ => match expr.to_string().as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
    }
}

/// Extract an array of strings from an HCL expression.
pub fn extract_string_array(expr: &Expression) -> Result<Vec<String>> {
    match expr {
        Expression::Array(items) => items
            .iter()
            .map(|item| {
                extract_string(item).ok_or_else(|| HclError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: format!("{:?}", item),
                })
            })
            .collect(),
        _ => Err(HclError::TypeMismatch {
            expected: "array".to_string(),
            actual: expr.to_string(),
        }),
    }
}

/// Extract enum validation values from a validation attribute.
pub fn extract_enum_validation(attr: &Attribute) -> Option<Vec<String>> {
    let expr_str = attr.expr().to_string();

    // Pattern: contains(["value1", "value2"], var.xyz)
    if expr_str.contains("contains(") && expr_str.contains('[') && expr_str.contains(']') {
        if let Some(start) = expr_str.find('[') {
            if let Some(end) = expr_str.find(']') {
                if start < end {
                    let values_str = &expr_str[start + 1..end];
                    return Some(
                        values_str
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                            .collect(),
                    );
                }
            }
        }
    }

    None
}

/// Extract attributes from a block as a HashMap.
pub fn extract_block_attributes(block: &Block) -> HashMap<String, HclValue> {
    let mut attributes = HashMap::new();

    for attr in block.body().attributes() {
        if let Ok(value) = expression_to_hcl_value(attr.expr()) {
            attributes.insert(attr.key().to_string(), value);
        }
    }

    attributes
}

/// Convert an HCL expression to an HclValue.
pub fn expression_to_hcl_value(expr: &Expression) -> Result<HclValue> {
    match expr {
        Expression::String(s) => Ok(HclValue::String(s.to_string())),
        Expression::Number(n) => Ok(HclValue::Number(*n)),
        Expression::Bool(b) => Ok(HclValue::Bool(*b)),
        Expression::Null => Ok(HclValue::Null),
        Expression::Array(items) => {
            let values: Result<Vec<_>> = items.iter().map(expression_to_hcl_value).collect();
            Ok(HclValue::Array(values?))
        }
        Expression::Object(obj) => {
            let mut map = HashMap::new();
            for (key, value) in obj.iter() {
                map.insert(key.as_str().to_string(), expression_to_hcl_value(value)?);
            }
            Ok(HclValue::Object(map))
        }
        _ => {
            // For complex expressions, try to extract as string
            if let Some(s) = extract_string(expr) {
                Ok(HclValue::String(s))
            } else {
                Ok(HclValue::String(expr.to_string()))
            }
        }
    }
}

/// Trait implementations for common types.
impl FromHclExpression for String {
    fn from_expression(expr: &Expression) -> Result<Self> {
        extract_string(expr).ok_or_else(|| HclError::TypeMismatch {
            expected: "string".to_string(),
            actual: expr.to_string(),
        })
    }
}

impl FromHclExpression for f64 {
    fn from_expression(expr: &Expression) -> Result<Self> {
        extract_number(expr).ok_or_else(|| HclError::TypeMismatch {
            expected: "number".to_string(),
            actual: expr.to_string(),
        })
    }
}

impl FromHclExpression for bool {
    fn from_expression(expr: &Expression) -> Result<Self> {
        extract_bool(expr).ok_or_else(|| HclError::TypeMismatch {
            expected: "boolean".to_string(),
            actual: expr.to_string(),
        })
    }
}

impl<T: FromHclExpression> FromHclExpression for Vec<T> {
    fn from_expression(expr: &Expression) -> Result<Self> {
        match expr {
            Expression::Array(items) => items.iter().map(T::from_expression).collect(),
            _ => Err(HclError::TypeMismatch {
                expected: "array".to_string(),
                actual: expr.to_string(),
            }),
        }
    }
}

/// Builder for extracting values from HCL structures.
pub struct HclExtractor<'a> {
    block: &'a Block,
}

impl<'a> HclExtractor<'a> {
    /// Create a new extractor for the given block.
    pub fn new(block: &'a Block) -> Self {
        Self { block }
    }

    /// Extract a required attribute.
    pub fn extract_required<T: FromHclExpression>(&self, key: &str) -> Result<T> {
        self.block
            .body()
            .attributes()
            .find(|attr| attr.key() == key)
            .ok_or_else(|| HclError::MissingField(key.to_string()))
            .and_then(|attr| T::from_expression(attr.expr()))
    }

    /// Extract an optional attribute.
    pub fn extract_optional<T: FromHclExpression>(&self, key: &str) -> Result<Option<T>> {
        self.block
            .body()
            .attributes()
            .find(|attr| attr.key() == key)
            .map(|attr| T::from_expression(attr.expr()))
            .transpose()
    }

    /// Extract an attribute with a default value.
    pub fn extract_with_default<T: FromHclExpression>(&self, key: &str, default: T) -> T {
        self.extract_optional(key)
            .unwrap_or(None)
            .unwrap_or(default)
    }
}
