use crate::application::{CreateSupplierRequest, UpdateSupplierRequest};
use crate::domain::{Supplier, SupplierId, OrganizationId};
use crate::errors::SupplierBusinessErrors;
use serde::{Deserialize, Serialize};
use shared::entities::{Address, AuditFields, BusinessError, default_user};
use shared::errors::api_error::ApiError;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSupplierDto {
    pub organization_id: String,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    #[serde(default = "default_user")]
    pub created_by: String,
}

impl TryFrom<CreateSupplierDto> for CreateSupplierRequest {
    type Error = ApiError;

    fn try_from(dto: CreateSupplierDto) -> Result<Self, Self::Error> {
        Ok(Self {
            organization_id: parse_id(&dto.organization_id)?,
            name: dto.name,
            nickname: dto.nickname,
            second_address: dto.second_address,
            created_by: dto.created_by,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSupplierDto {
    pub organization_id: String,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    #[serde(default = "default_user")]
    pub updated_by: String,
}

impl UpdateSupplierDto {
    pub fn into_request(self, id: SupplierId) -> Result<UpdateSupplierRequest, ApiError> {
        Ok(UpdateSupplierRequest {
            id,
            organization_id: parse_id(&self.organization_id)?,
            name: self.name,
            nickname: self.nickname,
            second_address: self.second_address,
            updated_by: self.updated_by,
        })
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FindSuppliersParams {
    pub name: Option<String>,
    pub organization_id: Option<String>,
    pub page: Option<u32>,
    #[serde(alias = "pageSize", alias = "limit")]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteSupplierParams {
    pub updated_by: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SupplierResponseDto {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    pub audit: Option<AuditFields>,
}

impl From<Supplier> for SupplierResponseDto {
    fn from(supplier: Supplier) -> Self {
        Self {
            id: supplier.id().to_string(),
            organization_id: supplier.organization_id().to_string(),
            name: supplier.name().to_string(),
            nickname: supplier.nickname().map(String::from),
            second_address: supplier.second_address().cloned(),
            audit: supplier.audit().cloned(),
        }
    }
}

pub fn parse_id(value: &str) -> Result<OrganizationId, ApiError> {
    value
        .parse::<OrganizationId>()
        .map_err(|_| ApiError::BadRequest(SupplierBusinessErrors::ORGANIZATION_ID_INVALID.into()))
}

pub fn map_api_error(error: BusinessError) -> ApiError {
    match error {
        BusinessError::NotFound(message) => ApiError::NotFound(message),
        other => other.into(),
    }
}
