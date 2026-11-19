use crate::v1::models::SupplierAddress;
use serde::{Deserialize, Serialize};

pub const EVENT_NAME: &str = "SupplierEvent";
/// Preserves the version value currently published in the AMQP header.
pub const EVENT_VERSION: i32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SupplierEventType {
    AddSupplier,
    UpdateSupplier,
    DeleteSupplier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupplierEvent {
    pub event_type: SupplierEventType,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_address: Option<SupplierAddress>,
}

impl SupplierEvent {
    pub fn add(
        id: impl Into<String>,
        organization_id: impl Into<String>,
        name: impl Into<String>,
        nickname: Option<String>,
        second_address: Option<SupplierAddress>,
    ) -> Self {
        Self {
            event_type: SupplierEventType::AddSupplier,
            id: id.into(),
            organization_id: Some(organization_id.into()),
            name: Some(name.into()),
            nickname,
            second_address,
        }
    }

    pub fn update(
        id: impl Into<String>,
        organization_id: impl Into<String>,
        name: impl Into<String>,
        nickname: Option<String>,
        second_address: Option<SupplierAddress>,
    ) -> Self {
        Self {
            event_type: SupplierEventType::UpdateSupplier,
            id: id.into(),
            organization_id: Some(organization_id.into()),
            name: Some(name.into()),
            nickname,
            second_address,
        }
    }

    pub fn delete(id: impl Into<String>) -> Self {
        Self {
            event_type: SupplierEventType::DeleteSupplier,
            id: id.into(),
            organization_id: None,
            name: None,
            nickname: None,
            second_address: None,
        }
    }
}
