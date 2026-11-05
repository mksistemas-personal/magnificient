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
                write!(f, "Invalid format. Expected: {expected}, received: {received}")
            }
            DocumentError::InvalidLength { expected, received } => {
                write!(f, "Invalid length. Expected: {expected} digits, received: {received}")
            }
            DocumentError::InvalidCheckDigits => {
                write!(f, "Invalid check digits")
            }
            DocumentError::EmptyDocument => {
                write!(f, "Document value cannot be empty")
            }
            DocumentError::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for DocumentError {}
