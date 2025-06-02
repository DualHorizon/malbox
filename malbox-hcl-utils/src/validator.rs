use crate::error::{HclError, Result};
use hcl::{Block, Body, Expression, Structure};
use std::collections::HashSet;

/// Trait for validating HCL structures.
pub trait HclValidator {
    fn validate(&self) -> Result<()>;
}

/// Validator for HCL bodies.
pub struct BodyValidator<'a> {
    body: &'a Body,
    rules: Vec<Box<dyn ValidationRule>>,
}

impl<'a> BodyValidator<'a> {
    /// Create a new validator for the given body.
    pub fn new(body: &'a Body) -> Self {
        Self {
            body,
            rules: Vec::new(),
        }
    }

    /// Add a validation rule.
    pub fn add_rule(mut self, rule: Box<dyn ValidationRule>) -> Self {
        self.rules.push(rule);
        self
    }

    /// Require specific blocks to exist.
    pub fn require_blocks(mut self, identifiers: &[&str]) -> Self {
        self.rules.push(Box::new(RequiredBlocks {
            identifiers: identifiers.iter().map(|s| s.to_string()).collect(),
        }));
        self
    }

    /// Require specific attributes to exist.
    pub fn require_attributes(mut self, keys: &[&str]) -> Self {
        self.rules.push(Box::new(RequiredAttributes {
            keys: keys.iter().map(|s| s.to_string()).collect(),
        }));
        self
    }

    /// Validate the body against all rules.
    pub fn validate(&self) -> Result<()> {
        for rule in &self.rules {
            rule.validate(self.body)?;
        }
        Ok(())
    }
}

/// Trait for validation rules.
pub trait ValidationRule {
    fn validate(&self, body: &Body) -> Result<()>;
}

/// Rule that requires specific blocks to exist.
struct RequiredBlocks {
    identifiers: HashSet<String>,
}

impl ValidationRule for RequiredBlocks {
    fn validate(&self, body: &Body) -> Result<()> {
        let mut found = HashSet::new();

        for structure in body.iter() {
            if let Structure::Block(block) = structure {
                found.insert(block.identifier().to_string());
            }
        }

        for required in &self.identifiers {
            if !found.contains(required) {
                return Err(HclError::MissingField(format!("block '{}'", required)));
            }
        }

        Ok(())
    }
}

/// Rule that requires specific attributes to exist.
struct RequiredAttributes {
    keys: HashSet<String>,
}

impl ValidationRule for RequiredAttributes {
    fn validate(&self, body: &Body) -> Result<()> {
        let mut found = HashSet::new();

        for attr in body.attributes() {
            found.insert(attr.key().to_string());
        }

        for required in &self.keys {
            if !found.contains(required) {
                return Err(HclError::MissingField(format!("attribute '{}'", required)));
            }
        }

        Ok(())
    }
}

/// Validator for HCL blocks.
pub struct BlockValidator<'a> {
    block: &'a Block,
}

impl<'a> BlockValidator<'a> {
    /// Create a new validator for the given block.
    pub fn new(block: &'a Block) -> Self {
        Self { block }
    }

    /// Validate that the block has the expected number of labels.
    pub fn validate_label_count(&self, expected: usize) -> Result<()> {
        let actual = self.block.labels().len();
        if actual != expected {
            return Err(HclError::ValidationError(format!(
                "Block '{}' expected {} labels, got {}",
                self.block.identifier(),
                expected,
                actual
            )));
        }
        Ok(())
    }

    /// Validate that the block has required attributes.
    pub fn validate_required_attributes(&self, required: &[&str]) -> Result<()> {
        let attributes: HashSet<_> = self
            .block
            .body()
            .attributes()
            .map(|attr| attr.key())
            .collect();

        for &attr in required {
            if !attributes.contains(attr) {
                return Err(HclError::MissingField(format!(
                    "Block '{}' missing required attribute '{}'",
                    self.block.identifier(),
                    attr
                )));
            }
        }

        Ok(())
    }

    /// Validate an attribute value against a predicate.
    pub fn validate_attribute<F>(&self, key: &str, predicate: F) -> Result<()>
    where
        F: Fn(&Expression) -> bool,
    {
        let attr = self
            .block
            .body()
            .attributes()
            .find(|attr| attr.key() == key)
            .ok_or_else(|| HclError::MissingField(key.to_string()))?;

        if !predicate(attr.expr()) {
            return Err(HclError::ValidationError(format!(
                "Attribute '{}' in block '{}' failed validation",
                key,
                self.block.identifier()
            )));
        }

        Ok(())
    }
}

/// Common validation functions.
pub mod validators {
    use super::*;

    /// Validate that a string is not empty.
    pub fn non_empty_string(expr: &Expression) -> bool {
        matches!(expr, Expression::String(s) if !s.is_empty())
    }

    /// Validate that a number is positive.
    pub fn positive_number(expr: &Expression) -> bool {
        matches!(expr, Expression::Number(n) if *n > 0.0)
    }

    /// Validate that a number is within a range.
    pub fn number_in_range(min: f64, max: f64) -> impl Fn(&Expression) -> bool {
        move |expr| matches!(expr, Expression::Number(n) if *n >= min && *n <= max)
    }

    /// Validate that a string matches a pattern.
    pub fn string_matches_pattern(pattern: &str) -> impl Fn(&Expression) -> bool {
        let pattern = pattern.to_string();
        move |expr| {
            if let Expression::String(s) = expr {
                s.contains(&pattern)
            } else {
                false
            }
        }
    }
}

/// Builder for creating complex validators.
pub struct ValidatorBuilder {
    rules: Vec<Box<dyn Fn(&Body) -> Result<()>>>,
}

impl ValidatorBuilder {
    /// Create a new validator builder.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a custom validation rule.
    pub fn add_rule<F>(mut self, rule: F) -> Self
    where
        F: Fn(&Body) -> Result<()> + 'static,
    {
        self.rules.push(Box::new(rule));
        self
    }

    /// Build the validator.
    pub fn build(self) -> impl Fn(&Body) -> Result<()> {
        move |body| {
            for rule in &self.rules {
                rule(body)?;
            }
            Ok(())
        }
    }
}

impl Default for ValidatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
