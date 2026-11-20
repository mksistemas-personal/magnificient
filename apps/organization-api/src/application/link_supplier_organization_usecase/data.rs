use crate::domain::{OrganizationId, SupplierId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkSupplierOrganizationRequest {
    pub organization_id: OrganizationId,
    pub supplier_id: SupplierId,
    pub created_by: String,
}

impl LinkSupplierOrganizationRequest {
    pub fn new(
        organization_id: OrganizationId,
        supplier_id: SupplierId,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            organization_id,
            supplier_id,
            created_by: created_by.into(),
        }
    }
}

pub struct InitialContextData {
    pub request: LinkSupplierOrganizationRequest,
}

pub struct ValidatedContextData {
    pub request: LinkSupplierOrganizationRequest,
}
