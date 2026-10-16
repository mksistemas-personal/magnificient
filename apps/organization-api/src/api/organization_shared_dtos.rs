use crate::domain::Organization;
use crate::errors::OrganizationBusinessErrors;
use serde::{Deserialize, Serialize};
use shared::entities::DocumentType;
use shared::errors::api_error::ApiError;

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

pub fn default_document_type() -> String {
    "CNPJ".to_string()
}

pub fn parse_document_type(doc_type_str: &str) -> Result<DocumentType, ApiError> {
    let doc_type = doc_type_str.trim().to_uppercase();
    if doc_type.is_empty() {
        return Err(ApiError::BadRequest(OrganizationBusinessErrors::ORG_DOCUMENT_INVALID.to_string()));
    }
    Ok(DocumentType::from_code(&doc_type, None))
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
