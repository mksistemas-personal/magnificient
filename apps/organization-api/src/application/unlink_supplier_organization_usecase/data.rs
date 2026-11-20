use crate::domain::SupplierId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlinkSupplierOrganizationRequest {
    pub supplier_id: SupplierId,
}

impl UnlinkSupplierOrganizationRequest {
    pub fn new(supplier_id: SupplierId) -> Self {
        Self { supplier_id }
    }
}

pub struct InitialContextData {
    pub request: UnlinkSupplierOrganizationRequest,
}

pub struct ValidatedContextData {
    pub request: UnlinkSupplierOrganizationRequest,
}
