use crate::domain::{Supplier, SupplierId, OrganizationId};
use shared::entities::Address;

#[derive(Debug, Clone)]
pub struct UpdateSupplierRequest {
    pub id: SupplierId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    pub updated_by: String,
}

pub struct InitialContextData {
    pub request: UpdateSupplierRequest,
}

pub struct ValidatedContextData {
    pub request: UpdateSupplierRequest,
}

pub struct AddressValidatedContextData {
    pub request: UpdateSupplierRequest,
}

pub struct LoadedContextData {
    pub request: UpdateSupplierRequest,
    pub supplier: Supplier,
}

pub struct ChangedContextData {
    pub supplier: Supplier,
}
