use std::fmt;
use std::num::ParseIntError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentifierError {
    InvalidFormat(String),
    ParseError(String),
    NonPositiveValue(u64),
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentifierError::InvalidFormat(msg) => write!(f, "Formato de identificador inválido: {msg}"),
            IdentifierError::ParseError(msg) => write!(f, "Erro ao converter identificador: {msg}"),
            IdentifierError::NonPositiveValue(val) => {
                write!(f, "Identificador deve ser um número positivo, recebido: {val}")
            }
        }
    }
}

impl std::error::Error for IdentifierError {}

impl From<ParseIntError> for IdentifierError {
    fn from(err: ParseIntError) -> Self {
        IdentifierError::ParseError(err.to_string())
    }
}
