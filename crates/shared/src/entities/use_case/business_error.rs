use crate::errors::{AddressError, DocumentError, IdentifierError};
use std::fmt;

/// Business error types used consistently across use cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusinessError {
    /// Entity or resource was not found.
    NotFound(String),

    /// Entity or resource already exists / conflict.
    AlreadyExists(String),

    /// Input data is invalid or inconsistent.
    InvalidData(String),

    /// A business validation rule failed.
    Validation(String),

    /// Action is not allowed or is unauthorized by business rules.
    NotAllowed(String),

    /// Internal execution error.
    Internal(String),
}

impl BusinessError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn already_exists(msg: impl Into<String>) -> Self {
        Self::AlreadyExists(msg.into())
    }

    pub fn invalid_data(msg: impl Into<String>) -> Self {
        Self::InvalidData(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn not_allowed(msg: impl Into<String>) -> Self {
        Self::NotAllowed(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn message(&self) -> &str {
        match self {
            Self::NotFound(msg)
            | Self::AlreadyExists(msg)
            | Self::InvalidData(msg)
            | Self::Validation(msg)
            | Self::NotAllowed(msg)
            | Self::Internal(msg) => msg.as_str(),
        }
    }
}

impl fmt::Display for BusinessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BusinessError::NotFound(msg) => write!(f, "Not found: {msg}"),
            BusinessError::AlreadyExists(msg) => write!(f, "Already exists: {msg}"),
            BusinessError::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
            BusinessError::Validation(msg) => write!(f, "Validation failed: {msg}"),
            BusinessError::NotAllowed(msg) => write!(f, "Operation not allowed: {msg}"),
            BusinessError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for BusinessError {}

impl From<DocumentError> for BusinessError {
    fn from(err: DocumentError) -> Self {
        BusinessError::InvalidData(format!("Invalid document: {err}"))
    }
}

impl From<AddressError> for BusinessError {
    fn from(err: AddressError) -> Self {
        BusinessError::InvalidData(err.to_string())
    }
}

impl From<IdentifierError> for BusinessError {
    fn from(err: IdentifierError) -> Self {
        BusinessError::InvalidData(format!("Invalid identifier: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_error_constructors_and_display() {
        let err = BusinessError::not_found("Organização 123");
        assert_eq!(err.message(), "Organização 123");
        assert_eq!(format!("{err}"), "Not found: Organização 123");

        let err = BusinessError::already_exists("CNPJ duplicado");
        assert_eq!(format!("{err}"), "Already exists: CNPJ duplicado");

        let err = BusinessError::invalid_data("Campo vazio");
        assert_eq!(format!("{err}"), "Invalid data: Campo vazio");
    }

    #[test]
    fn test_business_error_conversions() {
        let doc_err = DocumentError::EmptyDocument;
        let bus_err: BusinessError = doc_err.into();
        assert!(matches!(bus_err, BusinessError::InvalidData(_)));

        let addr_err = AddressError::EmptyField("street");
        let bus_err: BusinessError = addr_err.into();
        assert!(matches!(bus_err, BusinessError::InvalidData(_)));

        let id_err = IdentifierError::NonPositiveValue(0);
        let bus_err: BusinessError = id_err.into();
        assert!(matches!(bus_err, BusinessError::InvalidData(_)));
    }
}
