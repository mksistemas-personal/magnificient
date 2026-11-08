use crate::domain::OrganizationId;
use shared::types::PageRequest;
#[derive(Debug, Clone)]
pub struct FindClientsRequest {
    pub name: Option<String>,
    pub organization_id: Option<OrganizationId>,
    pub page_request: PageRequest,
}
pub struct InitialContextData {
    pub request: FindClientsRequest,
}
pub struct ValidatedContextData {
    pub request: FindClientsRequest,
}
