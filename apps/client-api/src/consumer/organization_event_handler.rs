mod data;
mod states;
mod steps;

use crate::api::AppState;
use crate::consumer::organization_event_handler::states::Initial;
use messaging::{MessageHandler, MessageHandlerError, MessageMetadata};
use organization_contracts::v1::OrganizationEvent;
use shared::entities::use_case::context::Context;

pub struct OrganizationEventHandler {
    state: AppState,
}

impl OrganizationEventHandler {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    async fn delete_clients_for_organization(
        &self,
        event: OrganizationEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        use steps::{DeleteOrganizationClients, ParseOrganizationId, ResolveTenant, ValidateEvent};

        Context::<Initial, _>::new(data::InitialContextData { event, metadata })
            .validate_event()?
            .resolve_tenant(&self.state)
            .await?
            .parse_organization_id()?
            .delete_organization_clients()
            .await
    }
}

#[async_trait::async_trait]
impl MessageHandler<OrganizationEvent> for OrganizationEventHandler {
    async fn handle(
        &self,
        event: OrganizationEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        use organization_contracts::v1::OrganizationEventType;

        if event.event_type == OrganizationEventType::DeleteOrganization {
            self.delete_clients_for_organization(event, metadata)
                .await?;
        }
        Ok(())
    }
}
