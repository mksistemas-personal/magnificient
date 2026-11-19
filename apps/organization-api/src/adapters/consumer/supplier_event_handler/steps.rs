use super::data::{
    EventValidatedContextData, InitialContextData, SupplierIdParsedContextData,
    TenantResolvedContextData,
};
use super::states::{EventValidated, Initial, SupplierIdParsed, TenantResolved};
use crate::adapters::api::AppState;
use crate::application::{LinkSupplierOrganizationRequest, UnlinkSupplierOrganizationRequest};
use crate::domain::{OrganizationId, SupplierId};
use crate::infrastructure::tenant_state;
use async_trait::async_trait;
use messaging::MessageHandlerError;
use shared::entities::Context;
use supplier_contracts::v1::SupplierEventType;

pub trait ValidateEvent {
    fn validate_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError>;
}

impl ValidateEvent for Context<Initial, InitialContextData> {
    fn validate_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError> {
        if self.data.event.event_type != SupplierEventType::AddSupplier {
            return Err(MessageHandlerError::permanent(
                "Expected an AddSupplier event",
            ));
        }
        let tenant = self.data.metadata.require_tenant()?.to_owned();

        if self.data.event.id.trim().is_empty() {
            return Err(MessageHandlerError::permanent(
                "SupplierEvent is missing id",
            ));
        }

        let organization_id = self
            .data
            .event
            .organization_id
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| {
                MessageHandlerError::permanent("SupplierEvent is missing organization_id")
            })?;

        Ok(Context::new(EventValidatedContextData {
            supplier_id: self.data.event.id,
            organization_id: Some(organization_id),
            tenant,
        }))
    }
}

pub trait ValidateDeleteEvent {
    fn validate_delete_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError>;
}

impl ValidateDeleteEvent for Context<Initial, InitialContextData> {
    fn validate_delete_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError> {
        if self.data.event.event_type != SupplierEventType::DeleteSupplier {
            return Err(MessageHandlerError::permanent(
                "Expected a DeleteSupplier event",
            ));
        }
        let tenant = self.data.metadata.require_tenant()?.to_owned();

        if self.data.event.id.trim().is_empty() {
            return Err(MessageHandlerError::permanent(
                "SupplierEvent is missing id",
            ));
        }

        Ok(Context::new(EventValidatedContextData {
            supplier_id: self.data.event.id,
            organization_id: None,
            tenant,
        }))
    }
}

#[async_trait]
pub trait ResolveTenant {
    async fn resolve_tenant(
        self,
        state: &AppState,
    ) -> Result<Context<TenantResolved, TenantResolvedContextData>, MessageHandlerError>;
}

#[async_trait]
impl ResolveTenant for Context<EventValidated, EventValidatedContextData> {
    async fn resolve_tenant(
        self,
        state: &AppState,
    ) -> Result<Context<TenantResolved, TenantResolvedContextData>, MessageHandlerError> {
        let tenant_state = tenant_state(state, &self.data.tenant).await?;
        Ok(Context::new(TenantResolvedContextData {
            supplier_id: self.data.supplier_id,
            organization_id: self.data.organization_id,
            tenant_state,
        }))
    }
}

pub trait ParseSupplierAndOrganizationId {
    fn parse_supplier_and_organization_id(
        self,
    ) -> Result<Context<SupplierIdParsed, SupplierIdParsedContextData>, MessageHandlerError>;
}

impl ParseSupplierAndOrganizationId for Context<TenantResolved, TenantResolvedContextData> {
    fn parse_supplier_and_organization_id(
        self,
    ) -> Result<Context<SupplierIdParsed, SupplierIdParsedContextData>, MessageHandlerError> {
        let supplier_id = self.data.supplier_id.parse::<SupplierId>().map_err(|error| {
            MessageHandlerError::permanent(format!(
                "Invalid supplier id '{}': {error}",
                self.data.supplier_id
            ))
        })?;
        let organization_id = match self.data.organization_id {
            Some(ref org_id) => Some(org_id.parse::<OrganizationId>().map_err(|error| {
                MessageHandlerError::permanent(format!(
                    "Invalid organization id '{}': {error}",
                    org_id
                ))
            })?),
            None => None,
        };
        Ok(Context::new(SupplierIdParsedContextData {
            supplier_id,
            organization_id,
            tenant_state: self.data.tenant_state,
        }))
    }
}

#[async_trait]
pub trait AddSupplierToOrganization {
    async fn link_supplier_organization(self) -> Result<(), MessageHandlerError>;
}

#[async_trait]
impl AddSupplierToOrganization for Context<SupplierIdParsed, SupplierIdParsedContextData> {
    async fn link_supplier_organization(self) -> Result<(), MessageHandlerError> {
        let organization_id = self.data.organization_id.ok_or_else(|| {
            MessageHandlerError::permanent("Missing organization_id for link_supplier_organization")
        })?;

        self.data
            .tenant_state
            .link_supplier_organization
            .execute(LinkSupplierOrganizationRequest::new(
                organization_id,
                self.data.supplier_id,
                "supplier-consumer",
            ))
            .await?;
        Ok(())
    }
}

#[async_trait]
pub trait RemoveSupplierFromOrganization {
    async fn unlink_supplier_organization(self) -> Result<(), MessageHandlerError>;
}

#[async_trait]
impl RemoveSupplierFromOrganization for Context<SupplierIdParsed, SupplierIdParsedContextData> {
    async fn unlink_supplier_organization(self) -> Result<(), MessageHandlerError> {
        self.data
            .tenant_state
            .unlink_supplier_organization
            .execute(UnlinkSupplierOrganizationRequest::new(
                self.data.supplier_id,
            ))
            .await?;
        Ok(())
    }
}
