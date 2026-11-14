use super::data::{
    ClientIdParsedContextData, EventValidatedContextData, InitialContextData,
    TenantResolvedContextData,
};
use super::states::{ClientIdParsed, EventValidated, Initial, TenantResolved};
use crate::adapters::api::AppState;
use crate::application::{LinkClientOrganizationRequest, UnlinkClientOrganizationRequest};
use crate::domain::{ClientId, OrganizationId};
use crate::infrastructure::tenant_state;
use async_trait::async_trait;
use client_contracts::v1::ClientEventType;
use messaging::MessageHandlerError;
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
        if self.data.event.event_type != ClientEventType::AddClient {
            return Err(MessageHandlerError::permanent(
                "Expected an AddClient event",
            ));
        }
        let tenant = self.data.metadata.require_tenant()?.to_owned();

        if self.data.event.id.trim().is_empty() {
            return Err(MessageHandlerError::permanent(
                "ClientEvent is missing id",
            ));
        }

        let organization_id = self
            .data
            .event
            .organization_id
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| {
                MessageHandlerError::permanent("ClientEvent is missing organization_id")
            })?;

        Ok(Context::new(EventValidatedContextData {
            client_id: self.data.event.id,
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
        if self.data.event.event_type != ClientEventType::DeleteClient {
            return Err(MessageHandlerError::permanent(
                "Expected a DeleteClient event",
            ));
        }
        let tenant = self.data.metadata.require_tenant()?.to_owned();

        if self.data.event.id.trim().is_empty() {
            return Err(MessageHandlerError::permanent(
                "ClientEvent is missing id",
            ));
        }

        Ok(Context::new(EventValidatedContextData {
            client_id: self.data.event.id,
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
            client_id: self.data.client_id,
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
        let client_id = self.data.client_id.parse::<ClientId>().map_err(|error| {
            MessageHandlerError::permanent(format!(
                "Invalid client id '{}': {error}",
                self.data.client_id
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
        Ok(Context::new(ClientIdParsedContextData {
            client_id,
            organization_id,
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
        let organization_id = self.data.organization_id.ok_or_else(|| {
            MessageHandlerError::permanent("Missing organization_id for link_client_organization")
        })?;

        self.data
            .tenant_state
            .link_client_organization
            .execute(LinkClientOrganizationRequest::new(
                organization_id,
                self.data.client_id,
                "client-consumer",
            ))
            .await?;
        Ok(())
    }
}

#[async_trait]
pub trait RemoveClientFromOrganization {
    async fn unlink_client_organization(self) -> Result<(), MessageHandlerError>;
}

#[async_trait]
impl RemoveClientFromOrganization for Context<ClientIdParsed, ClientIdParsedContextData> {
    async fn unlink_client_organization(self) -> Result<(), MessageHandlerError> {
        self.data
            .tenant_state
            .unlink_client_organization
            .execute(UnlinkClientOrganizationRequest::new(
                self.data.client_id,
            ))
            .await?;
        Ok(())
    }
}
