use crate::application::{CreateClientRequest, UpdateClientRequest};
use crate::domain::{Client, ClientId, OrganizationId};
use crate::errors::ClientBusinessErrors;
use serde::{Deserialize, Serialize};
use shared::entities::converter::serde_functions::default_user;
use shared::entities::{Address, AuditFields};
use shared::errors::api_error::ApiError;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateClientDto {
    pub organization_id: String,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    #[serde(default = "default_user")]
    pub created_by: String,
}

impl TryFrom<CreateClientDto> for CreateClientRequest {
    type Error = ApiError;

    fn try_from(dto: CreateClientDto) -> Result<Self, Self::Error> {
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
pub struct UpdateClientDto {
    pub organization_id: String,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    #[serde(default = "default_user")]
    pub updated_by: String,
}

impl UpdateClientDto {
    pub fn into_request(self, id: ClientId) -> Result<UpdateClientRequest, ApiError> {
        Ok(UpdateClientRequest {
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
pub struct FindClientsParams {
    pub name: Option<String>,
    pub organization_id: Option<String>,
    pub page: Option<u32>,
    #[serde(alias = "pageSize", alias = "limit")]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteClientParams {
    pub updated_by: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientResponseDto {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    pub audit: Option<AuditFields>,
}

impl From<Client> for ClientResponseDto {
    fn from(client: Client) -> Self {
        Self {
            id: client.id.to_string(),
            organization_id: client.organization_id.to_string(),
            name: client.name,
            nickname: client.nickname,
            second_address: client.second_address,
            audit: client.audit,
        }
    }
}

pub fn parse_id(value: &str) -> Result<OrganizationId, ApiError> {
    value
        .parse::<OrganizationId>()
        .map_err(|_| ApiError::BadRequest(ClientBusinessErrors::ORGANIZATION_ID_INVALID.into()))
}

pub fn map_api_error(error: shared::entities::BusinessError) -> ApiError {
    match error {
        shared::entities::BusinessError::NotFound(message) => ApiError::NotFound(message),
        other => other.into(),
    }
}
