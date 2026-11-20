use crate::application::{CreateProductRequest, UpdateProductRequest};
use crate::domain::{Product, ProductId, OrganizationId};
use crate::errors::ProductBusinessErrors;
use serde::{Deserialize, Serialize};
use shared::entities::{AuditFields, BusinessError, default_user};
use shared::errors::api_error::ApiError;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductDto {
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub price: Option<f64>,
    #[serde(default = "default_user")]
    pub created_by: String,
}

impl TryFrom<CreateProductDto> for CreateProductRequest {
    type Error = ApiError;

    fn try_from(dto: CreateProductDto) -> Result<Self, Self::Error> {
        Ok(Self {
            organization_id: parse_id(&dto.organization_id)?,
            name: dto.name,
            description: dto.description,
            sku: dto.sku,
            price: dto.price,
            created_by: dto.created_by,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductDto {
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub price: Option<f64>,
    #[serde(default = "default_user")]
    pub updated_by: String,
}

impl UpdateProductDto {
    pub fn into_request(self, id: ProductId) -> Result<UpdateProductRequest, ApiError> {
        Ok(UpdateProductRequest {
            id,
            organization_id: parse_id(&self.organization_id)?,
            name: self.name,
            description: self.description,
            sku: self.sku,
            price: self.price,
            updated_by: self.updated_by,
        })
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FindProductsParams {
    pub name: Option<String>,
    pub organization_id: Option<String>,
    pub page: Option<u32>,
    #[serde(alias = "pageSize", alias = "limit")]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteProductParams {
    pub updated_by: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductResponseDto {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub price: Option<f64>,
    pub audit: Option<AuditFields>,
}

impl From<Product> for ProductResponseDto {
    fn from(product: Product) -> Self {
        Self {
            id: product.id().to_string(),
            organization_id: product.organization_id().to_string(),
            name: product.name().to_string(),
            description: product.description().map(String::from),
            sku: product.sku().map(String::from),
            price: product.price(),
            audit: product.audit().cloned(),
        }
    }
}

pub fn parse_id(value: &str) -> Result<OrganizationId, ApiError> {
    value
        .parse::<OrganizationId>()
        .map_err(|_| ApiError::BadRequest(ProductBusinessErrors::ORGANIZATION_ID_INVALID.into()))
}

pub fn map_api_error(error: BusinessError) -> ApiError {
    match error {
        BusinessError::NotFound(message) => ApiError::NotFound(message),
        other => other.into(),
    }
}
