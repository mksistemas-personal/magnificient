use crate::domain::{ClientId, OrganizationId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkClientOrganizationRequest {
    pub organization_id: OrganizationId,
    pub client_id: ClientId,
    pub created_by: String,
}

impl LinkClientOrganizationRequest {
    pub fn new(
        organization_id: OrganizationId,
        client_id: ClientId,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            organization_id,
            client_id,
            created_by: created_by.into(),
        }
    }
}

pub struct InitialContextData {
    pub request: LinkClientOrganizationRequest,
}

pub struct ValidatedContextData {
    pub request: LinkClientOrganizationRequest,
}
