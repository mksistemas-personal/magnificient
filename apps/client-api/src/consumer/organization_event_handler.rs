use crate::api::AppState;
use crate::application::{DeleteClientRequest, FindClientsRequest};
use crate::domain::OrganizationId;
use crate::infrastructure::tenant_state;
use messaging::{MessageHandler, MessageHandlerError, MessageMetadata};
use organization_contracts::v1::{OrganizationEvent, OrganizationEventType};
use shared::entities::BusinessError;
use shared::types::PageRequest;
use tracing::error;

pub struct OrganizationEventHandler {
    state: AppState,
}

impl OrganizationEventHandler {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    async fn delete_clients_for_organization(
        &self,
        event: &OrganizationEvent,
        metadata: &MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        let tenant = metadata
            .tenant
            .as_deref()
            .filter(|tenant| !tenant.is_empty())
            .ok_or_else(|| {
                MessageHandlerError::permanent("OrganizationEvent is missing tenant metadata")
            })?;

        let tenant_state = tenant_state(&self.state, tenant)
            .await
            .map_err(map_tenant_error)?;

        let organization_id = event.id.parse::<OrganizationId>().map_err(|error| {
            MessageHandlerError::permanent(format!(
                "Invalid organization id '{}': {error}",
                event.id
            ))
        })?;

        // Page through matching clients, then invoke the existing delete use case
        // for each client so its validation, audit fields, and UnitOfWork remain active.
        loop {
            let page = tenant_state
                .find_clients
                .execute(FindClientsRequest {
                    name: None,
                    organization_id: Some(organization_id),
                    page_request: PageRequest::new(1, 100),
                })
                .await
                .map_err(map_delete_error)?;

            let clients = page.content;
            if clients.is_empty() {
                break;
            }
            for client in clients {
                tenant_state
                    .delete_client
                    .execute(DeleteClientRequest {
                        id: client.id,
                        updated_by: "organization-api".to_owned(),
                    })
                    .await
                    .map_err(map_delete_error)?;
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl MessageHandler<OrganizationEvent> for OrganizationEventHandler {
    async fn handle(
        &self,
        event: OrganizationEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        if event.event_type == OrganizationEventType::DeleteOrganization {
            self.delete_clients_for_organization(&event, &metadata)
                .await?;
        }
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

fn map_delete_error(error: BusinessError) -> MessageHandlerError {
    match error {
        BusinessError::NotFound(_) => MessageHandlerError::permanent(error.to_string()),
        BusinessError::Internal(_) => {
            error!(error = %error, "Client deletion failed while processing OrganizationEvent");
            MessageHandlerError::retryable(error.to_string())
        }
        _ => MessageHandlerError::permanent(error.to_string()),
    }
}
