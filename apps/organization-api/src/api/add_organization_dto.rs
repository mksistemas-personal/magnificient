use crate::api::organization_shared_dto::default_document_type;
use crate::api::organization_shared_dto::default_user;
use crate::api::organization_shared_dto::parse_document_type;
use crate::application::AddOrganizationRequest;
use serde::{Deserialize, Serialize};
use shared::entities::Address;
use shared::errors::api_error::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateOrganizationDto {
    pub name: String,
    pub document_value: String,
    #[serde(default = "default_document_type")]
    pub document_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    #[serde(default = "default_user")]
    pub created_by: String,
}

impl TryFrom<CreateOrganizationDto> for AddOrganizationRequest {
    type Error = ApiError;

    fn try_from(dto: CreateOrganizationDto) -> Result<Self, Self::Error> {
        let document_type = parse_document_type(&dto.document_type)?;

        Ok(AddOrganizationRequest {
            name: dto.name,
            document_value: dto.document_value,
            document_type,
            address: dto.address,
            created_by: dto.created_by,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::organization_shared_dto::OrganizationResponseDto;
    use crate::domain::Organization;
    use shared::entities::{AuditFields, Document, DocumentType};
    use tsid::TSID;

    #[test]
    fn test_dto_try_into_request() {
        let dto = CreateOrganizationDto {
            name: "Empresa Teste".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: "CNPJ".to_string(),
            address: None,
            created_by: "admin".to_string(),
        };

        let req: Result<AddOrganizationRequest, _> = dto.try_into();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.name, "Empresa Teste");
        assert_eq!(req.document_type, DocumentType::Cnpj);
        assert_eq!(req.address, None);
    }

    #[test]
    fn test_dto_try_into_request_with_address() {
        let address = Address::new(
            "Av. Paulista",
            "1000",
            "Conjunto 101",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "01310-100",
            "Brasil",
            "BR",
            "",
        );
        let dto = CreateOrganizationDto {
            name: "Empresa Teste".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: "CNPJ".to_string(),
            address: Some(address.clone()),
            created_by: "admin".to_string(),
        };

        let req: Result<AddOrganizationRequest, _> = dto.try_into();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.address, Some(address));
    }

    #[test]
    fn test_organization_response_conversion() {
        use crate::domain::organization::OrganizationId;
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let audit = AuditFields::created("admin");
        let tsid = TSID::from(10u64);
        let org = Organization::new(OrganizationId::new(tsid), "Org ABC", doc).with_audit(audit);

        let response = OrganizationResponseDto::from(org);
        assert_eq!(response.id, tsid.to_string());
        assert_eq!(response.name, "Org ABC");
        assert_eq!(response.document.document_type, "CNPJ");
        assert_eq!(response.document.country.as_deref(), Some("BR"));
        assert!(response.audit.is_some());
        assert_eq!(response.audit.unwrap().created_by, "admin");
    }
}
