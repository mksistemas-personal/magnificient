use crate::domain::ProductId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlinkProductOrganizationRequest {
    pub product_id: ProductId,
}

impl UnlinkProductOrganizationRequest {
    pub fn new(product_id: ProductId) -> Self {
        Self { product_id }
    }
}

pub struct InitialContextData {
    pub request: UnlinkProductOrganizationRequest,
}

pub struct ValidatedContextData {
    pub request: UnlinkProductOrganizationRequest,
}
