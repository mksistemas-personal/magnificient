use crate::domain::ClientId;
#[derive(Debug, Clone, Copy)]
pub struct GetClientByIdRequest {
    pub id: ClientId,
}
pub struct InitialContextData {
    pub request: GetClientByIdRequest,
}
pub struct ValidatedContextData {
    pub request: GetClientByIdRequest,
}
