use crate::domain::{Client, ClientId, OrganizationId};
use shared::entities::Address;

#[derive(Debug, Clone)]
pub struct UpdateClientRequest {
    pub id: ClientId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    pub updated_by: String,
}

pub struct InitialContextData {
    pub request: UpdateClientRequest,
}

pub struct ValidatedContextData {
    pub request: UpdateClientRequest,
}

pub struct AddressValidatedContextData {
    pub request: UpdateClientRequest,
}

pub struct LoadedContextData {
    pub request: UpdateClientRequest,
    pub client: Client,
}

pub struct ChangedContextData {
    pub client: Client,
}
