use crate::domain::{OrganizationId, ProductId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkProductOrganizationRequest {
    pub organization_id: OrganizationId,
    pub product_id: ProductId,
    pub created_by: String,
}

impl LinkProductOrganizationRequest {
    pub fn new(
        organization_id: OrganizationId,
        product_id: ProductId,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            organization_id,
            product_id,
            created_by: created_by.into(),
        }
    }
}

pub struct InitialContextData {
    pub request: LinkProductOrganizationRequest,
}

pub struct ValidatedContextData {
    pub request: LinkProductOrganizationRequest,
}
