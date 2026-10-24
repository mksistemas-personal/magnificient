use crate::domain::{Organization, OrganizationId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOrganizationRequest {
    pub name: String,
    pub updated_by: String,
    pub organization_id: OrganizationId,
}

pub struct InitialContextData {
    pub request: UpdateOrganizationRequest,
}

pub struct ValidatedContextData {
    pub request: UpdateOrganizationRequest,
}

pub struct LoadedContextData {
    pub request: UpdateOrganizationRequest,
    pub organization: Organization,
}

pub struct ChangedContextData {
    pub organization: Organization,
}
