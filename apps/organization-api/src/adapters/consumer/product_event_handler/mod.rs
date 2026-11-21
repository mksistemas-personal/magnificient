pub mod data;
pub mod states;
pub mod steps;

use crate::adapters::api::AppState;
use crate::adapters::consumer::product_event_handler::states::Initial;
use async_trait::async_trait;
use messaging::{MessageHandler, MessageHandlerError, MessageMetadata};
use product_contracts::v1::{ProductEvent, ProductEventType};
use shared::entities::Context;

pub struct ProductEventHandler {
    app_state: AppState,
}

impl ProductEventHandler {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn handle_add_product(
        &self,
        event: ProductEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        use steps::{
            AddProductToOrganization, ParseProductAndOrganizationId, ResolveTenant, ValidateEvent,
        };

        Context::<Initial, _>::new(data::InitialContextData { event, metadata })
            .validate_event()?
            .resolve_tenant(&self.app_state)
            .await?
            .parse_product_and_organization_id()?
            .link_product_organization()
            .await
    }

    async fn handle_delete_product(
        &self,
        event: ProductEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        use steps::{
            ParseProductAndOrganizationId, RemoveProductFromOrganization, ResolveTenant,
            ValidateDeleteEvent,
        };

        Context::<Initial, _>::new(data::InitialContextData { event, metadata })
            .validate_delete_event()?
            .resolve_tenant(&self.app_state)
            .await?
            .parse_product_and_organization_id()?
            .unlink_product_organization()
            .await
    }
}

#[async_trait]
impl MessageHandler<ProductEvent> for ProductEventHandler {
    async fn handle(
        &self,
        event: ProductEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        match event.event_type {
            ProductEventType::AddProduct => {
                self.handle_add_product(event, metadata).await?;
            }
            ProductEventType::DeleteProduct => {
                self.handle_delete_product(event, metadata).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
