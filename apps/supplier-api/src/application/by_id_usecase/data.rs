use crate::domain::SupplierId;
#[derive(Debug, Clone, Copy)]
pub struct GetSupplierByIdRequest {
    pub id: SupplierId,
}
pub struct InitialContextData {
    pub request: GetSupplierByIdRequest,
}
pub struct ValidatedContextData {
    pub request: GetSupplierByIdRequest,
}
