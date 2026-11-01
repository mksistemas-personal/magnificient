use crate::domain::{Organization, OrganizationId};
use serde::{Deserialize, Serialize};
use shared::entities::{Address, Document};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<AddressEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentEvent {
    pub value: String,
    pub document_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressEvent {
    pub street: String,
    pub number: String,
    pub complement: String,
    pub neighborhood: String,
    pub city: String,
    pub state: String,
    pub state_code: String,
    pub postal_code: String,
    pub country: String,
    pub country_code: String,
    pub reference: String,
}

impl From<&Address> for AddressEvent {
    fn from(address: &Address) -> Self {
        Self {
            street: address.street.clone(),
            number: address.number.clone(),
            complement: address.complement.clone(),
            neighborhood: address.neighborhood.clone(),
            city: address.city.clone(),
            state: address.state.clone(),
            state_code: address.state_code.clone(),
            postal_code: address.postal_code.clone(),
            country: address.country.clone(),
            country_code: address.country_code.clone(),
            reference: address.reference.clone(),
        }
    }
}

impl OrganizationEvent {
    pub fn add(organization: Organization) -> Self {
        Self {
            event_type: OrganizationEventType::AddOrganization,
            id: organization.id.to_string(),
            name: Some(organization.name.to_string()),
            document: Some(organization.document.into()),
            address: organization.address.as_ref().map(|address| address.into()),
        }
    }

    pub fn update(organization: Organization) -> Self {
        Self {
            event_type: OrganizationEventType::UpdateOrganization,
            id: organization.id.to_string(),
            name: Some(organization.name),
            document: Some(organization.document.into()),
            address: organization.address.as_ref().map(|address| address.into()),
        }
    }

    pub fn delete(id: OrganizationId) -> Self {
        Self {
            event_type: OrganizationEventType::DeleteOrganization,
            id: id.to_string(),
            name: None,
            document: None,
            address: None,
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
