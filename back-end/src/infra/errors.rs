use std::fmt;

#[derive(Debug)]
pub enum InfraError {
    InternalServerError,
    NotFound,
}

pub fn adapt_infra_error<T: Error>(error: T) -> InfraError {
    error.as_infra_error()
}

// NOTE: Implement the Errors correctly, currently this is only a temporary solution.

impl fmt::Display for InfraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfraError::NotFound => write!(f, "Not found"),
            InfraError::InternalServerError => write!(f, "Internal server error"),
        }
    }
}

pub trait Error {
    fn as_infra_error(&self) -> InfraError;
}

impl Error for sqlx::error::Error {
    fn as_infra_error(&self) -> InfraError {
        match self {
            sqlx::Error::RowNotFound => InfraError::NotFound,
            _ => InfraError::InternalServerError,
        }
    }
}
