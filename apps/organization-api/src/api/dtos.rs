use crate::api::errors::ApiError;
use crate::application::add_organization::AddOrganizationRequest;
use crate::domain::organization::Organization;
use serde::{Deserialize, Serialize};
use shared::entities::DocumentType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateOrganizationDto {
    #[serde(default)]
    pub id: Option<i64>,
    pub name: String,
    pub document_value: String,
    #[serde(default = "default_document_type")]
    pub document_type: String,
    pub created_by: String,
}

fn default_document_type() -> String {
    "CNPJ".to_string()
}

pub fn parse_document_type(doc_type_str: &str) -> Result<DocumentType, ApiError> {
    match doc_type_str.trim().to_uppercase().as_str() {
        "CNPJ" => Ok(DocumentType::Cnpj),
        "CPF" => Ok(DocumentType::Cpf),
        "CUIT" => Ok(DocumentType::Cuit),
        "CUIL" => Ok(DocumentType::Cuil),
        "DNI" => Ok(DocumentType::Dni),
        "PASSPORT" | "PASSAPORTE" => Ok(DocumentType::Passport),
        other if !other.is_empty() => Ok(DocumentType::Other {
            name: other.to_string(),
            country: None,
        }),
        _ => Err(ApiError::BadRequest("Tipo de documento não informado ou inválido".to_string())),
    }
}

impl TryFrom<CreateOrganizationDto> for AddOrganizationRequest {
    type Error = ApiError;

    fn try_from(dto: CreateOrganizationDto) -> Result<Self, Self::Error> {
        let document_type = parse_document_type(&dto.document_type)?;
        Ok(AddOrganizationRequest {
            name: dto.name,
            document_value: dto.document_value,
            document_type,
            created_by: dto.created_by,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationResponseDto {
    pub id: String,
    pub name: String,
    pub document: DocumentResponseDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit: Option<AuditResponseDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentResponseDto {
    pub value: String,
    pub sanitized_value: String,
    pub formatted: String,
    pub document_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditResponseDto {
    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
}

impl From<Organization> for OrganizationResponseDto {
    fn from(org: Organization) -> Self {
        Self {
            id: org.id().to_string(),
            name: org.name().to_string(),
            document: DocumentResponseDto {
                value: org.document().value().to_string(),
                sanitized_value: org.document().sanitized_value(),
                formatted: org.document().formatted(),
                document_type: org.document().document_type().code().to_string(),
                country: org.document().country().map(|c| c.code().to_string()),
            },
            audit: org.audit.as_ref().map(|a| AuditResponseDto {
                created_by: a.created_by().to_string(),
                created_at: a.created_at().to_rfc3339(),
                updated_by: a.updated_by().to_string(),
                updated_at: a.updated_at().to_rfc3339(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::entities::{AuditFields, Document};
    use tsid::TSID;

    #[test]
    fn test_dto_try_into_request() {
        let dto = CreateOrganizationDto {
            id: Some(1),
            name: "Empresa Teste".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: "CNPJ".to_string(),
            created_by: "admin".to_string(),
        };

        let req: Result<AddOrganizationRequest, _> = dto.try_into();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.name, "Empresa Teste");
        assert_eq!(req.document_type, DocumentType::Cnpj);
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
