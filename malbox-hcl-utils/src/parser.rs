use crate::{
    error::{HclError, Result},
    types::{HclBlock, HclDocument, HclValue},
};
use hcl::{Body, Structure};
use std::collections::HashMap;
use tracing::debug;

/// Parse HCL content into a Body.
pub fn parse(content: &str) -> Result<Body> {
    hcl::from_str(content).map_err(HclError::ParseError)
}

/// Parse HCL content into a high-level document structure.
pub fn parse_document(content: &str) -> Result<HclDocument> {
    let body = parse(content)?;
    body_to_document(&body)
}

/// Convert an HCL Body to a high-level document structure.
pub fn body_to_document(body: &Body) -> Result<HclDocument> {
    let mut blocks = Vec::new();
    let mut attributes = HashMap::new();

    for structure in body.iter() {
        match structure {
            Structure::Block(block) => {
                blocks.push(block_to_hcl_block(block)?);
            }
            Structure::Attribute(attr) => {
                let value = expression_to_value(attr.expr())?;
                attributes.insert(attr.key().to_string(), value);
            }
        }
    }

    Ok(HclDocument { blocks, attributes })
}

/// Convert an HCL Block to a high-level block structure.
fn block_to_hcl_block(block: &hcl::Block) -> Result<HclBlock> {
    let identifier = block.identifier().to_string();
    let labels = block
        .labels()
        .iter()
        .map(|l| l.as_str().to_string())
        .collect();

    let mut attributes = HashMap::new();
    let mut nested_blocks = Vec::new();

    for attr in block.body().attributes() {
        let value = expression_to_value(attr.expr())?;
        attributes.insert(attr.key().to_string(), value);
    }

    for structure in block.body().iter() {
        if let Structure::Block(nested) = structure {
            nested_blocks.push(block_to_hcl_block(nested)?);
        }
    }

    Ok(HclBlock {
        identifier,
        labels,
        attributes,
        nested_blocks,
    })
}

/// Convert an HCL Expression to a high-level value.
fn expression_to_value(expr: &hcl::Expression) -> Result<HclValue> {
    use hcl::Expression;

    match expr {
        Expression::String(s) => Ok(HclValue::String(s.to_string())),
        Expression::Number(n) => Ok(HclValue::Number(*n)),
        Expression::Bool(b) => Ok(HclValue::Bool(*b)),
        Expression::Null => Ok(HclValue::Null),
        Expression::Array(items) => {
            let values: Result<Vec<_>> = items.iter().map(expression_to_value).collect();
            Ok(HclValue::Array(values?))
        }
        Expression::Object(obj) => {
            let mut map = HashMap::new();
            for (key, value) in obj.iter() {
                map.insert(key.as_str().to_string(), expression_to_value(value)?);
            }
            Ok(HclValue::Object(map))
        }
        _ => {
            // For other expression types, convert to string representation
            let s = expr.to_string();
            debug!("Converting complex expression to string: {}", s);
            Ok(HclValue::String(s))
        }
    }
}

/// Parse HCL content and extract specific structures.
pub struct HclParser {
    content: String,
    body: Option<Body>,
}

impl HclParser {
    /// Create a new parser with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            body: None,
        }
    }

    /// Parse the content.
    pub fn parse(&mut self) -> Result<&Body> {
        if self.body.is_none() {
            self.body = Some(parse(&self.content)?);
        }
        Ok(self.body.as_ref().unwrap())
    }

    /// Get all blocks with a specific identifier.
    pub fn get_blocks(&mut self, identifier: &str) -> Result<Vec<&hcl::Block>> {
        let body = self.parse()?;
        let blocks: Vec<_> = body
            .iter()
            .filter_map(|structure| {
                if let Structure::Block(block) = structure {
                    if block.identifier() == identifier {
                        Some(block)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        Ok(blocks)
    }

    /// Get all attributes at the root level.
    pub fn get_attributes(&mut self) -> Result<HashMap<String, &hcl::Expression>> {
        let body = self.parse()?;
        let mut attributes = HashMap::new();

        for attr in body.attributes() {
            attributes.insert(attr.key().to_string(), attr.expr());
        }

        Ok(attributes)
    }

    /// Find a specific block by identifier and labels.
    pub fn find_block(&mut self, identifier: &str, labels: &[&str]) -> Result<Option<&hcl::Block>> {
        let blocks = self.get_blocks(identifier)?;

        for block in blocks {
            let block_labels: Vec<_> = block.labels().iter().map(|l| l.as_str()).collect();
            if block_labels.as_slice() == labels {
                return Ok(Some(block));
            }
        }

        Ok(None)
    }
}
