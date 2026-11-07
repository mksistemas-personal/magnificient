use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    EmptyField(&'static str),
    InvalidPostalCode {},
    InvalidStateCode { country: String, received: String },
    InvalidCountryCode(String),
    Custom(String),
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressError::EmptyField(field) => {
                write!(f, "address.field.required.{field}'")
            }
            AddressError::InvalidPostalCode {} => {
                write!(f, "address.invalid.postal.code")
            }
            AddressError::InvalidStateCode {
                country: _,
                received: _,
            } => {
                write!(f, "address.invalid.state.province")
            }
            AddressError::InvalidCountryCode(_) => {
                write!(f, "address.invalid.country.code")
            }
            AddressError::Custom(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AddressError {}
