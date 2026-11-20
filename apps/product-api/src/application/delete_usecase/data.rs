use crate::domain::ProductId;

#[derive(Debug, Clone)]
pub struct DeleteProductRequest {
    pub id: ProductId,
    pub updated_by: String,
}
pub struct InitialContextData {
    pub request: DeleteProductRequest,
}
pub struct ValidatedContextData {
    pub request: DeleteProductRequest,
}
