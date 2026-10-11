use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    InvalidFormat {
        expected: &'static str,
        received: String,
    },
    InvalidLength {
        expected: usize,
        received: usize,
    },
    InvalidCheckDigits,
    EmptyDocument,
    Custom(String),
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentError::InvalidFormat { expected, received } => {
                write!(f, "Formato inválido. Esperado: {expected}, recebido: {received}")
            }
            DocumentError::InvalidLength { expected, received } => {
                write!(f, "Tamanho inválido. Esperado: {expected} dígitos, recebido: {received}")
            }
            DocumentError::InvalidCheckDigits => {
                write!(f, "Dígitos verificadores inválidos")
            }
            DocumentError::EmptyDocument => {
                write!(f, "O valor do documento não pode ser vazio")
            }
            DocumentError::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DocumentError {}
