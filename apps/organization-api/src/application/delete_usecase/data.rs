use crate::domain::OrganizationId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteOrganizationRequest {
    pub organization_id: OrganizationId,
    pub updated_by: String,
}

impl DeleteOrganizationRequest {
    pub fn new(organization_id: OrganizationId, updated_by: impl Into<String>) -> Self {
        Self {
            organization_id,
            updated_by: updated_by.into(),
        }
    }
}

pub struct InitialContextData {
    pub request: DeleteOrganizationRequest,
}

pub struct ValidatedContextData {
    pub request: DeleteOrganizationRequest,
}
