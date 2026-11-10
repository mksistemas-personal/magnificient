use crate::domain::Client;
use serde::{Deserialize, Serialize};
use shared::entities::Address;

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
    pub second_address: Option<ClientAddressEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientAddressEvent {
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

impl From<&Address> for ClientAddressEvent {
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
