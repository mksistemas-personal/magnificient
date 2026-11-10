use crate::domain::Client;
use serde::{Deserialize, Serialize};
use shared::entities::address::address_event::AddressEvent;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ClientEventType {
    AddClient,
    UpdateClient,
    DeleteClient,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientEvent {
    pub event_type: ClientEventType,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_address: Option<AddressEvent>,
}

impl ClientEvent {
    pub fn add(client: &Client) -> Self {
        Self::from_client(ClientEventType::AddClient, client)
    }

    pub fn update(client: &Client) -> Self {
        Self::from_client(ClientEventType::UpdateClient, client)
    }

    pub fn delete(id: impl ToString) -> Self {
        Self {
            event_type: ClientEventType::DeleteClient,
            id: id.to_string(),
            organization_id: None,
            name: None,
            nickname: None,
            second_address: None,
        }
    }

    fn from_client(event_type: ClientEventType, client: &Client) -> Self {
        Self {
            event_type,
            id: client.id.to_string(),
            organization_id: Some(client.organization_id.to_string()),
            name: Some(client.name.clone()),
            nickname: client.nickname.clone(),
            second_address: client.second_address.as_ref().map(Into::into),
        }
    }
}
