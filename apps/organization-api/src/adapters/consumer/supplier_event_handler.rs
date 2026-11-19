use async_trait::async_trait;
use messaging::{MessageHandler, MessageHandlerError, MessageMetadata};
use supplier_contracts::v1::{SupplierEvent, SupplierEventType};
use tracing::info;

pub struct SupplierEventHandler;

impl SupplierEventHandler {
    pub fn new(_app_state: crate::adapters::api::AppState) -> Self {
        Self
    }

    fn validate_event(
        &self,
        event: &SupplierEvent,
        metadata: &MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        metadata.require_tenant().map(|_| ())?;
        if event.id.trim().is_empty() {
            return Err(MessageHandlerError::permanent(
                "SupplierEvent is missing id",
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl MessageHandler<SupplierEvent> for SupplierEventHandler {
    async fn handle(
        &self,
        event: SupplierEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        self.validate_event(&event, &metadata)?;

        let tenant = metadata.tenant.as_deref().unwrap_or("");

        match event.event_type {
            SupplierEventType::AddSupplier => {
                info!(
                    supplier_id = %event.id,
                    tenant = %tenant,
                    "Received AddSupplier event"
                );
            }
            SupplierEventType::UpdateSupplier => {
                info!(
                    supplier_id = %event.id,
                    tenant = %tenant,
                    "Received UpdateSupplier event"
                );
            }
            SupplierEventType::DeleteSupplier => {
                info!(
                    supplier_id = %event.id,
                    tenant = %tenant,
                    "Received DeleteSupplier event"
                );
            }
        }
        Ok(())
    }
}
