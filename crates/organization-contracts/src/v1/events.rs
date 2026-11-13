use crate::v1::models::{OrganizationAddress, OrganizationDocument};
use serde::{Deserialize, Serialize};

pub const EVENT_NAME: &str = "OrganizationEvent";
/// Preserves the version value currently published in the AMQP header.
pub const EVENT_VERSION: i32 = 100;

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
    pub document: Option<OrganizationDocument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<OrganizationAddress>,
}

impl OrganizationEvent {
    pub fn add(
        id: impl Into<String>,
        name: impl Into<String>,
        document: OrganizationDocument,
        address: Option<OrganizationAddress>,
    ) -> Self {
        Self {
            event_type: OrganizationEventType::AddOrganization,
            id: id.into(),
            name: Some(name.into()),
            document: Some(document),
            address,
        }
    }

    pub fn update(
        id: impl Into<String>,
        name: impl Into<String>,
        document: OrganizationDocument,
        address: Option<OrganizationAddress>,
    ) -> Self {
        Self {
            event_type: OrganizationEventType::UpdateOrganization,
            id: id.into(),
            name: Some(name.into()),
            document: Some(document),
            address,
        }
    }

    pub fn delete(id: impl Into<String>) -> Self {
        Self {
            event_type: OrganizationEventType::DeleteOrganization,
            id: id.into(),
            name: None,
            document: None,
            address: None,
        }
    }
}
