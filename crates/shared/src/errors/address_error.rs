use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    EmptyField(&'static str),
    InvalidPostalCode {
        country: String,
        received: String,
        expected: &'static str,
    },
    InvalidStateCode {
        country: String,
        received: String,
    },
    InvalidCountryCode(String),
    Custom(String),
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressError::EmptyField(field) => {
                write!(f, "O campo '{field}' é obrigatório e não pode ser vazio")
            }
            AddressError::InvalidPostalCode {
                country,
                received,
                expected,
            } => {
                write!(
                    f,
                    "Código postal inválido para o país {country}: '{received}'. Esperado: {expected}"
                )
            }
            AddressError::InvalidStateCode { country, received } => {
                write!(
                    f,
                    "Código de estado/província inválido para o país {country}: '{received}'"
                )
            }
            AddressError::InvalidCountryCode(code) => {
                write!(f, "Código de país inválido: '{code}'")
            }
            AddressError::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AddressError {}
