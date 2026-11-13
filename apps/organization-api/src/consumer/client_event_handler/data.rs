use crate::api::AppState;
use crate::domain::{ClientId, OrganizationId};
use client_contracts::v1::ClientEvent;
use messaging::MessageMetadata;

pub struct InitialContextData {
    pub event: ClientEvent,
    pub metadata: MessageMetadata,
}

pub struct EventValidatedContextData {
    pub client_id: String,
    pub organization_id: Option<String>,
    pub tenant: String,
}

pub struct TenantResolvedContextData {
    pub client_id: String,
    pub organization_id: Option<String>,
    pub tenant_state: AppState,
}

pub struct ClientIdParsedContextData {
    pub client_id: ClientId,
    pub organization_id: Option<OrganizationId>,
    pub tenant_state: AppState,
}
