use crate::adapters::api::AppState;
use crate::domain::{OrganizationId, ProductId};
use messaging::MessageMetadata;
use product_contracts::v1::ProductEvent;

pub struct InitialContextData {
    pub event: ProductEvent,
    pub metadata: MessageMetadata,
}

pub struct EventValidatedContextData {
    pub product_id: String,
    pub organization_id: Option<String>,
    pub tenant: String,
}

pub struct TenantResolvedContextData {
    pub product_id: String,
    pub organization_id: Option<String>,
    pub tenant_state: AppState,
}

pub struct ProductIdParsedContextData {
    pub product_id: ProductId,
    pub organization_id: Option<OrganizationId>,
    pub tenant_state: AppState,
}
