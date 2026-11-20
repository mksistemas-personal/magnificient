use crate::adapters::api::AppState;
use crate::domain::{OrganizationId, SupplierId};
use messaging::MessageMetadata;
use supplier_contracts::v1::SupplierEvent;

pub struct InitialContextData {
    pub event: SupplierEvent,
    pub metadata: MessageMetadata,
}

pub struct EventValidatedContextData {
    pub supplier_id: String,
    pub organization_id: Option<String>,
    pub tenant: String,
}

pub struct TenantResolvedContextData {
    pub supplier_id: String,
    pub organization_id: Option<String>,
    pub tenant_state: AppState,
}

pub struct SupplierIdParsedContextData {
    pub supplier_id: SupplierId,
    pub organization_id: Option<OrganizationId>,
    pub tenant_state: AppState,
}
