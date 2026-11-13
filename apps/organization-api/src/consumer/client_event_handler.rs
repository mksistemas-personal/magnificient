pub mod data;
pub mod states;
pub mod steps;

use crate::api::AppState;
use crate::consumer::client_event_handler::states::Initial;
use async_trait::async_trait;
use client_contracts::v1::{ClientEvent, ClientEventType};
use messaging::{MessageHandler, MessageHandlerError, MessageMetadata};
use shared::entities::use_case::context::Context;

pub struct ClientEventHandler {
    app_state: AppState,
}

impl ClientEventHandler {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn handle_add_client(
        &self,
        event: ClientEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        use steps::{AddClientToOrganization, ParseClientAndOrganizationId, ResolveTenant, ValidateEvent};

        Context::<Initial, _>::new(data::InitialContextData { event, metadata })
            .validate_event()?
            .resolve_tenant(&self.app_state)
            .await?
            .parse_client_and_organization_id()?
            .link_client_organization()
            .await
    }
}

#[async_trait]
impl MessageHandler<ClientEvent> for ClientEventHandler {
    async fn handle(
        &self,
        event: ClientEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        if event.event_type == ClientEventType::AddClient {
            self.handle_add_client(event, metadata).await?;
        }
        Ok(())
    }
}
