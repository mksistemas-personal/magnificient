use messaging::{MessageHandler, MessageHandlerError, MessageMetadata};
use organization_contracts::v1::{OrganizationEvent, OrganizationEventType};
use tracing::info;

pub struct OrganizationEventHandler;

#[async_trait::async_trait]
impl MessageHandler<OrganizationEvent> for OrganizationEventHandler {
    async fn handle(
        &self,
        event: OrganizationEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        process_organization_event(event, metadata);
        Ok(())
    }
}

/// The Client API currently has no Organization read model to update; this
/// handler dispatches and records the event metadata for operational visibility.
fn process_organization_event(event: OrganizationEvent, metadata: MessageMetadata) {
    let event_action = match event.event_type {
        OrganizationEventType::AddOrganization => "add",
        OrganizationEventType::UpdateOrganization => "update",
        OrganizationEventType::DeleteOrganization => "delete",
    };
    info!(
        organization_id = %event.id,
        event_action,
        event_type = %metadata.event_type,
        event_version = metadata.version,
        message_id = metadata.message_id.as_deref().unwrap_or("unknown"),
        tenant = metadata.tenant.as_deref().unwrap_or("unknown"),
        "Processed OrganizationEvent"
    );
}
