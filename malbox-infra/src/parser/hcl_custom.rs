use crate::error::{Error, Result};
use hcl::{Attribute, Body, Expression};

pub fn parse(content: &str) -> Result<Body> {
    hcl::from_str(content).map_err(|e| Error::HclParse(e))
}

pub fn extract_string_value(expr: &Expression) -> Option<String> {
    match expr {
        Expression::String(s) => Some(s.to_string()),
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

pub fn parse_enum_validation(attr: &Attribute) -> Option<Vec<String>> {
    let expr_str = attr.expr().to_string();

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
