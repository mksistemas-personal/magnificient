use super::data::{
    EventValidatedContextData, InitialContextData, OrganizationIdParsedContextData,
    TenantResolvedContextData,
};
use super::states::{EventValidated, Initial, OrganizationIdParsed, TenantResolved};
use crate::adapters::api::AppState;
use crate::application::DeleteOrganizationClientsRequest;
use crate::domain::OrganizationId;
use crate::infrastructure::tenant_state;
use async_trait::async_trait;
use messaging::MessageHandlerError;
use organization_contracts::v1::OrganizationEventType;
use shared::entities::use_case::context::Context;

pub trait ValidateEvent {
    fn validate_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError>;
}

impl ValidateEvent for Context<Initial, InitialContextData> {
    fn validate_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError> {
        if self.data.event.event_type != OrganizationEventType::DeleteOrganization {
            return Err(MessageHandlerError::permanent(
                "Expected a DeleteOrganization event",
            ));
        }
        let tenant = self.data.metadata.require_tenant()?.to_owned();

        Ok(Context::new(EventValidatedContextData {
            event_id: self.data.event.id,
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
            event_id: self.data.event_id,
            tenant_state,
        }))
    }
}

pub trait ParseOrganizationId {
    fn parse_organization_id(
        self,
    ) -> Result<Context<OrganizationIdParsed, OrganizationIdParsedContextData>, MessageHandlerError>;
}

impl ParseOrganizationId for Context<TenantResolved, TenantResolvedContextData> {
    fn parse_organization_id(
        self,
    ) -> Result<Context<OrganizationIdParsed, OrganizationIdParsedContextData>, MessageHandlerError>
    {
        let organization_id = self
            .data
            .event_id
            .parse::<OrganizationId>()
            .map_err(|error| {
                MessageHandlerError::permanent(format!(
                    "Invalid organization id '{}': {error}",
                    self.data.event_id
                ))
            })?;
        Ok(Context::new(OrganizationIdParsedContextData {
            organization_id,
            tenant_state: self.data.tenant_state,
        }))
    }
}

#[async_trait]
pub trait DeleteOrganizationClients {
    async fn delete_organization_clients(self) -> Result<(), MessageHandlerError>;
}

#[async_trait]
impl DeleteOrganizationClients for Context<OrganizationIdParsed, OrganizationIdParsedContextData> {
    async fn delete_organization_clients(self) -> Result<(), MessageHandlerError> {
        self.data
            .tenant_state
            .delete_organization_clients
            .execute(DeleteOrganizationClientsRequest {
                organization_id: self.data.organization_id,
                updated_by: "organization-api".to_owned(),
            })
            .await?;
        Ok(())
    }
}
