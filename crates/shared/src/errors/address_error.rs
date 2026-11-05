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
                write!(f, "Field '{field}' is required and cannot be empty")
            }
            AddressError::InvalidPostalCode {
                country,
                received,
                expected,
            } => {
                write!(
                    f,
                    "Invalid postal code for country {country}: '{received}'. Expected: {expected}"
                )
            }
            AddressError::InvalidStateCode { country, received } => {
                write!(
                    f,
                    "Invalid state/province code for country {country}: '{received}'"
                )
            }
            AddressError::InvalidCountryCode(code) => {
                write!(f, "Invalid country code: '{code}'")
            }
            AddressError::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AddressError {}
