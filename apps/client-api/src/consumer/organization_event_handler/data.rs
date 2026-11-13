use crate::api::AppState;
use crate::domain::OrganizationId;
use messaging::MessageMetadata;
use organization_contracts::v1::OrganizationEvent;

pub struct InitialContextData {
    pub event: OrganizationEvent,
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

pub struct OrganizationIdParsedContextData {
    pub organization_id: OrganizationId,
    pub tenant_state: AppState,
}
