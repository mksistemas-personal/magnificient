use crate::domain::{Supplier, OrganizationId};
use shared::entities::Address;

#[derive(Debug, Clone)]
pub struct CreateSupplierRequest {
    pub organization_id: OrganizationId,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    pub created_by: String,
}

pub struct InitialContextData {
    pub request: CreateSupplierRequest,
}

pub struct ValidatedContextData {
    pub request: CreateSupplierRequest,
}

pub struct AddressValidatedContextData {
    pub request: CreateSupplierRequest,
    pub organization_id: OrganizationId,
}

pub struct BuiltContextData {
    pub supplier: Supplier,
}
