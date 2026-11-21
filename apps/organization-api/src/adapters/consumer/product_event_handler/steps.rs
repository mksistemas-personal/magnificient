use super::data::{
    EventValidatedContextData, InitialContextData, ProductIdParsedContextData,
    TenantResolvedContextData,
};
use super::states::{EventValidated, Initial, ProductIdParsed, TenantResolved};
use crate::adapters::api::AppState;
use crate::application::{LinkProductOrganizationRequest, UnlinkProductOrganizationRequest};
use crate::domain::{OrganizationId, ProductId};
use crate::infrastructure::tenant_state;
use async_trait::async_trait;
use messaging::MessageHandlerError;
use product_contracts::v1::ProductEventType;
use shared::entities::Context;

pub trait ValidateEvent {
    fn validate_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError>;
}

impl ValidateEvent for Context<Initial, InitialContextData> {
    fn validate_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError> {
        if self.data.event.event_type != ProductEventType::AddProduct {
            return Err(MessageHandlerError::permanent(
                "Expected an AddProduct event",
            ));
        }
        let tenant = self.data.metadata.require_tenant()?.to_owned();

        if self.data.event.id.trim().is_empty() {
            return Err(MessageHandlerError::permanent(
                "ProductEvent is missing id",
            ));
        }

        let organization_id = self
            .data
            .event
            .organization_id
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| {
                MessageHandlerError::permanent("ProductEvent is missing organization_id")
            })?;

        Ok(Context::new(EventValidatedContextData {
            product_id: self.data.event.id,
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
        if self.data.event.event_type != ProductEventType::DeleteProduct {
            return Err(MessageHandlerError::permanent(
                "Expected a DeleteProduct event",
            ));
        }
        let tenant = self.data.metadata.require_tenant()?.to_owned();

        if self.data.event.id.trim().is_empty() {
            return Err(MessageHandlerError::permanent(
                "ProductEvent is missing id",
            ));
        }

        Ok(Context::new(EventValidatedContextData {
            product_id: self.data.event.id,
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
            product_id: self.data.product_id,
            organization_id: self.data.organization_id,
            tenant_state,
        }))
    }
}

pub trait ParseProductAndOrganizationId {
    fn parse_product_and_organization_id(
        self,
    ) -> Result<Context<ProductIdParsed, ProductIdParsedContextData>, MessageHandlerError>;
}

impl ParseProductAndOrganizationId for Context<TenantResolved, TenantResolvedContextData> {
    fn parse_product_and_organization_id(
        self,
    ) -> Result<Context<ProductIdParsed, ProductIdParsedContextData>, MessageHandlerError> {
        let product_id = self.data.product_id.parse::<ProductId>().map_err(|error| {
            MessageHandlerError::permanent(format!(
                "Invalid product id '{}': {error}",
                self.data.product_id
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
        Ok(Context::new(ProductIdParsedContextData {
            product_id,
            organization_id,
            tenant_state: self.data.tenant_state,
        }))
    }
}

#[async_trait]
pub trait AddProductToOrganization {
    async fn link_product_organization(self) -> Result<(), MessageHandlerError>;
}

#[async_trait]
impl AddProductToOrganization for Context<ProductIdParsed, ProductIdParsedContextData> {
    async fn link_product_organization(self) -> Result<(), MessageHandlerError> {
        let organization_id = self.data.organization_id.ok_or_else(|| {
            MessageHandlerError::permanent("Missing organization_id for link_product_organization")
        })?;

        self.data
            .tenant_state
            .link_product_organization
            .execute(LinkProductOrganizationRequest::new(
                organization_id,
                self.data.product_id,
                "product-consumer",
            ))
            .await?;
        Ok(())
    }
}

#[async_trait]
pub trait RemoveProductFromOrganization {
    async fn unlink_product_organization(self) -> Result<(), MessageHandlerError>;
}

#[async_trait]
impl RemoveProductFromOrganization for Context<ProductIdParsed, ProductIdParsedContextData> {
    async fn unlink_product_organization(self) -> Result<(), MessageHandlerError> {
        self.data
            .tenant_state
            .unlink_product_organization
            .execute(UnlinkProductOrganizationRequest::new(
                self.data.product_id,
            ))
            .await?;
        Ok(())
    }
}
