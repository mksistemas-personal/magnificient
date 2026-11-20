use crate::domain::{Product, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateProductRequest {
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub price: Option<f64>,
    pub created_by: String,
}

pub struct InitialContextData {
    pub request: CreateProductRequest,
}

pub struct ValidatedContextData {
    pub request: CreateProductRequest,
}

pub struct BuiltContextData {
    pub product: Product,
}
