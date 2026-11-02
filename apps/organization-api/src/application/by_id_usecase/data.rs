use crate::domain::OrganizationId;
use tsid::TSID;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindOrganizationByIdRequest {
    pub organization_id: OrganizationId,
}

impl FindOrganizationByIdRequest {
    pub fn new(organization_id: OrganizationId) -> Self {
        Self { organization_id }
    }
}

impl From<OrganizationId> for FindOrganizationByIdRequest {
    fn from(organization_id: OrganizationId) -> Self {
        Self { organization_id }
    }
}

impl From<TSID> for FindOrganizationByIdRequest {
    fn from(tsid: TSID) -> Self {
        Self {
            organization_id: OrganizationId::new(tsid),
        }
    }
}

pub type GetOrganizationByIdRequest = FindOrganizationByIdRequest;

pub struct InitialContextData {
    pub request: FindOrganizationByIdRequest,
}

pub struct ValidatedContextData {
    pub request: FindOrganizationByIdRequest,
}
