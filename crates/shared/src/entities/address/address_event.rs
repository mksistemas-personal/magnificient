use crate::entities::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressEvent {
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

impl From<&Address> for AddressEvent {
    fn from(address: &Address) -> Self {
        Self {
            street: address.street.clone(),
            number: address.number.clone(),
            complement: address.complement.clone(),
            neighborhood: address.neighborhood.clone(),
            city: address.city.clone(),
            state: address.state.clone(),
            state_code: address.state_code.clone(),
            postal_code: address.postal_code.clone(),
            country: address.country.clone(),
            country_code: address.country_code.clone(),
            reference: address.reference.clone(),
        }
    }
}
