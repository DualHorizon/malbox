use thiserror::Error;

#[derive(Error, Debug)]
pub enum HclError {
    #[error("HCL parse error: {0}")]
    ParseError(#[from] hcl::Error),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Extraction error: {0}")]
    ExtractionError(String),
}

pub type Result<T> = std::result::Result<T, HclError>;
