use serde::{Deserialize, Serialize};

/// Address representation shared by public event contracts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Address {
    pub street: String,
    pub number: String,
    pub complement: String,
    pub neighborhood: String,
    pub city: String,
    pub state: String,
    pub state_code: String,
    pub postal_code: String,
    pub country: String,
    pub country_code: String,
    pub reference: String,
}
