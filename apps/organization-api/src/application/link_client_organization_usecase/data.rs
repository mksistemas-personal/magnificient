use crate::domain::ClientId;

#[derive(Debug, Clone)]
pub struct LinkClientOrganizationRequest {
    pub client_id: ClientId,
    pub updated_by: String,
}
