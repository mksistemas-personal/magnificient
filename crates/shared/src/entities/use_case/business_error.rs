use std::fmt;
use crate::errors::{DocumentError, IdentifierError};

/// Enum de erros de negócio para uso padronizado em casos de uso (Use Cases).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusinessError {
    /// Entidade ou recurso não encontrado
    NotFound(String),

    /// Entidade ou recurso já existente / conflito
    AlreadyExists(String),

    /// Dados de entrada inválidos ou inconsistentes
    InvalidData(String),

    /// Falha de regra de validação de negócio
    Validation(String),

    /// Ação não permitida ou operação não autorizada pelas regras de negócio
    NotAllowed(String),

    /// Erro interno de execução
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
            BusinessError::NotFound(msg) => write!(f, "Não encontrado: {msg}"),
            BusinessError::AlreadyExists(msg) => write!(f, "Já existe: {msg}"),
            BusinessError::InvalidData(msg) => write!(f, "Dados inválidos: {msg}"),
            BusinessError::Validation(msg) => write!(f, "Regra violada: {msg}"),
            BusinessError::NotAllowed(msg) => write!(f, "Operação não permitida: {msg}"),
            BusinessError::Internal(msg) => write!(f, "Erro interno: {msg}"),
        }
    }
}

impl std::error::Error for BusinessError {}

impl From<DocumentError> for BusinessError {
    fn from(err: DocumentError) -> Self {
        BusinessError::InvalidData(format!("Documento inválido: {err}"))
    }
}

impl From<IdentifierError> for BusinessError {
    fn from(err: IdentifierError) -> Self {
        BusinessError::InvalidData(format!("Identificador inválido: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_error_constructors_and_display() {
        let err = BusinessError::not_found("Organização 123");
        assert_eq!(err.message(), "Organização 123");
        assert_eq!(format!("{err}"), "Não encontrado: Organização 123");

        let err = BusinessError::already_exists("CNPJ duplicado");
        assert_eq!(format!("{err}"), "Já existe: CNPJ duplicado");

        let err = BusinessError::invalid_data("Campo vazio");
        assert_eq!(format!("{err}"), "Dados inválidos: Campo vazio");
    }

    #[test]
    fn test_business_error_conversions() {
        let doc_err = DocumentError::EmptyDocument;
        let bus_err: BusinessError = doc_err.into();
        assert!(matches!(bus_err, BusinessError::InvalidData(_)));

        let id_err = IdentifierError::NonPositiveValue(0);
        let bus_err: BusinessError = id_err.into();
        assert!(matches!(bus_err, BusinessError::InvalidData(_)));
    }
}