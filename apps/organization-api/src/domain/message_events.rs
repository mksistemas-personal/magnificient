use crate::domain::{Organization, OrganizationId};
use serde::{Deserialize, Serialize};
use shared::entities::Document;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum OrganizationEventType {
    AddOrganization,
    UpdateOrganization,
    DeleteOrganization,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationEvent {
    pub event_type: OrganizationEventType,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<DocumentEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentEvent {
    pub value: String,
    pub document_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

impl OrganizationEvent {
    pub fn add(organization: Organization) -> Self {
        Self {
            event_type: OrganizationEventType::AddOrganization,
            id: organization.id.to_string(),
            name: Some(organization.name.to_string()),
            document: Some(organization.document.into()),
        }
    }

    pub fn update(organization: Organization) -> Self {
        Self {
            event_type: OrganizationEventType::UpdateOrganization,
            id: organization.id.to_string(),
            name: Some(organization.name),
            document: Some(organization.document.into()),
        }
    }

    pub fn delete(id: OrganizationId) -> Self {
        Self {
            event_type: OrganizationEventType::DeleteOrganization,
            id: id.to_string(),
            name: None,
            document: None,
        }
    }
}

impl From<Document> for DocumentEvent {
    fn from(document: Document) -> Self {
        Self {
            value: document.value.to_string(),
            document_type: document.document_type.code().to_string(),
            country: document.country().map(|c| c.code().to_string()),
        }
    }
}
