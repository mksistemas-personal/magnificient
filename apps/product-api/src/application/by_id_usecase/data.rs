use crate::domain::ProductId;
#[derive(Debug, Clone, Copy)]
pub struct GetProductByIdRequest {
    pub id: ProductId,
}
pub struct InitialContextData {
    pub request: GetProductByIdRequest,
}
pub struct ValidatedContextData {
    pub request: GetProductByIdRequest,
}
