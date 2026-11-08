use crate::domain::{Client, OrganizationId};
use shared::entities::Address;

#[derive(Debug, Clone)]
pub struct CreateClientRequest {
    pub organization_id: OrganizationId,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    pub created_by: String,
}

pub struct InitialContextData {
    pub request: CreateClientRequest,
}

pub struct ValidatedContextData {
    pub request: CreateClientRequest,
}

pub struct AddressValidatedContextData {
    pub request: CreateClientRequest,
    pub organization_id: OrganizationId,
}

pub struct BuiltContextData {
    pub client: Client,
}
