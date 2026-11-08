use crate::domain::ClientId;

#[derive(Debug, Clone)]
pub struct DeleteClientRequest {
    pub id: ClientId,
    pub updated_by: String,
}
pub struct InitialContextData {
    pub request: DeleteClientRequest,
}
pub struct ValidatedContextData {
    pub request: DeleteClientRequest,
}
