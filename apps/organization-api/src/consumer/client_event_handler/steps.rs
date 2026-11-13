use super::data::{
    ClientIdParsedContextData, EventValidatedContextData, InitialContextData,
    TenantResolvedContextData,
};
use super::states::{ClientIdParsed, EventValidated, Initial, TenantResolved};
use crate::api::AppState;
use crate::application::LinkClientOrganizationRequest;
use crate::domain::{ClientId, OrganizationId};
use crate::infrastructure::tenant_state;
use async_trait::async_trait;
use client_contracts::v1::ClientEventType;
use messaging::MessageHandlerError;
use shared::entities::use_case::context::Context;
use tracing::error;

pub trait ValidateEvent {
    fn validate_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError>;
}

impl ValidateEvent for Context<Initial, InitialContextData> {
    fn validate_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError> {
        if self.data.event.event_type != ClientEventType::AddClient {
            return Err(MessageHandlerError::permanent(
                "Expected an AddClient event",
            ));
        }
        let tenant = self
            .data
            .metadata
            .tenant
            .filter(|tenant| !tenant.trim().is_empty())
            .ok_or_else(|| {
                MessageHandlerError::permanent("ClientEvent is missing tenant metadata")
            })?;

        let organization_id = self
            .data
            .event
            .organization_id
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| {
                MessageHandlerError::permanent("ClientEvent is missing organization_id")
            })?;

        Ok(Context::new(EventValidatedContextData {
            event_id: self.data.event.id,
            organization_id,
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
        let tenant_state = tenant_state(state, &self.data.tenant)
            .await
            .map_err(map_tenant_error)?;
        Ok(Context::new(TenantResolvedContextData {
            event_id: self.data.event_id,
            organization_id: self.data.organization_id,
            tenant_state,
        }))
    }
}

pub trait ParseClientAndOrganizationId {
    fn parse_client_and_organization_id(
        self,
    ) -> Result<Context<ClientIdParsed, ClientIdParsedContextData>, MessageHandlerError>;
}

impl ParseClientAndOrganizationId for Context<TenantResolved, TenantResolvedContextData> {
    fn parse_client_and_organization_id(
        self,
    ) -> Result<Context<ClientIdParsed, ClientIdParsedContextData>, MessageHandlerError> {
        let client_id = self.data.event_id.parse::<ClientId>().map_err(|error| {
            MessageHandlerError::permanent(format!(
                "Invalid client id '{}': {error}",
                self.data.event_id
            ))
        })?;
        let organization_id = self
            .data
            .organization_id
            .parse::<OrganizationId>()
            .map_err(|error| {
                MessageHandlerError::permanent(format!(
                    "Invalid organization id '{}': {error}",
                    self.data.organization_id
                ))
            })?;
        Ok(Context::new(ClientIdParsedContextData {
            organization_id,
            client_id,
            tenant_state: self.data.tenant_state,
        }))
    }
}

#[async_trait]
pub trait AddClientToOrganization {
    async fn link_client_organization(self) -> Result<(), MessageHandlerError>;
}

#[async_trait]
impl AddClientToOrganization for Context<ClientIdParsed, ClientIdParsedContextData> {
    async fn link_client_organization(self) -> Result<(), MessageHandlerError> {
        self.data
            .tenant_state
            .link_client_organization
            .execute(LinkClientOrganizationRequest::new(
                self.data.organization_id,
                self.data.client_id,
                "client-consumer",
            ))
            .await
            .map_err(map_link_error)?;
        Ok(())
    }
}

fn map_tenant_error(error: shared::errors::api_error::ApiError) -> MessageHandlerError {
    match error {
        shared::errors::api_error::ApiError::Internal(_) => {
            MessageHandlerError::retryable(error.to_string())
        }
        _ => MessageHandlerError::permanent(error.to_string()),
    }
}

fn map_link_error(error: shared::entities::BusinessError) -> MessageHandlerError {
    match error {
        shared::entities::BusinessError::NotFound(_) => {
            MessageHandlerError::permanent(error.to_string())
        }
        shared::entities::BusinessError::AlreadyExists(_) => {
            MessageHandlerError::permanent(error.to_string())
        }
        shared::entities::BusinessError::Internal(_) => {
            error!(error = %error, "Link client to organization failed while processing ClientEvent");
            MessageHandlerError::retryable(error.to_string())
        }
        _ => MessageHandlerError::permanent(error.to_string()),
    }
}
