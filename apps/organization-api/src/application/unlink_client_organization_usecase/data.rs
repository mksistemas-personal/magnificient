use crate::domain::ClientId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlinkClientOrganizationRequest {
    pub client_id: ClientId,
}

impl UnlinkClientOrganizationRequest {
    pub fn new(client_id: ClientId) -> Self {
        Self { client_id }
    }
}

pub struct InitialContextData {
    pub request: UnlinkClientOrganizationRequest,
}

pub struct ValidatedContextData {
    pub request: UnlinkClientOrganizationRequest,
}
