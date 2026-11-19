use crate::domain::SupplierId;

#[derive(Debug, Clone)]
pub struct DeleteSupplierRequest {
    pub id: SupplierId,
    pub updated_by: String,
}
pub struct InitialContextData {
    pub request: DeleteSupplierRequest,
}
pub struct ValidatedContextData {
    pub request: DeleteSupplierRequest,
}
