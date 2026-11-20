use crate::domain::{Product, ProductId, OrganizationId};

#[derive(Debug, Clone)]
pub struct UpdateProductRequest {
    pub id: ProductId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub price: Option<f64>,
    pub updated_by: String,
}

pub struct InitialContextData {
    pub request: UpdateProductRequest,
}

pub struct ValidatedContextData {
    pub request: UpdateProductRequest,
}

pub struct LoadedContextData {
    pub request: UpdateProductRequest,
    pub product: Product,
}

pub struct ChangedContextData {
    pub product: Product,
}
