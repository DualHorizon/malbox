use hcl::{Block, Expression};
use std::collections::HashMap;

/// Represents an extracted HCL value.
#[derive(Debug, Clone, PartialEq)]
pub enum HclValue {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<HclValue>),
    Object(HashMap<String, HclValue>),
    Null,
}

impl HclValue {
    /// Try to convert to a string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            HclValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to convert to a number.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            HclValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to convert to a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            HclValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to convert to an array.
    pub fn as_array(&self) -> Option<&Vec<HclValue>> {
        match self {
            HclValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Try to convert to an object.
    pub fn as_object(&self) -> Option<&HashMap<String, HclValue>> {
        match self {
            HclValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

/// Trait for types that can be extracted from HCL expressions.
pub trait FromHclExpression: Sized {
    fn from_expression(expr: &Expression) -> crate::Result<Self>;
}

/// Trait for types that can be extracted from HCL blocks.
pub trait FromHclBlock: Sized {
    fn from_block(block: &Block) -> crate::Result<Self>;
}

/// Represents a parsed HCL document structure.
#[derive(Debug, Clone)]
pub struct HclDocument {
    pub blocks: Vec<HclBlock>,
    pub attributes: HashMap<String, HclValue>,
}

/// Represents a parsed HCL block.
#[derive(Debug, Clone)]
pub struct HclBlock {
    pub identifier: String,
    pub labels: Vec<String>,
    pub attributes: HashMap<String, HclValue>,
    pub nested_blocks: Vec<HclBlock>,
}
