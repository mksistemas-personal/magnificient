use crate::api::AppState;
use crate::domain::ClientId;
use client_contracts::v1::ClientEvent;
use messaging::MessageMetadata;

pub struct InitialContextData {
    pub event: ClientEvent,
    pub metadata: MessageMetadata,
}

pub struct EventValidatedContextData {
    pub event_id: String,
    pub tenant: String,
}

pub struct TenantResolvedContextData {
    pub event_id: String,
    pub tenant_state: AppState,
}

pub struct ClientIdParsedContextData {
    pub client_id: ClientId,
    pub tenant_state: AppState,
}
