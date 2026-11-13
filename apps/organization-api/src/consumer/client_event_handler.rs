pub mod data;
pub mod states;
pub mod steps;

use crate::api::AppState;
use client_contracts::v1::ClientEvent;
use messaging::{MessageHandlerError, MessageMetadata};

pub struct ClientEventHandler {
    app_state: AppState,
}

impl ClientEventHandler {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn handle_client_event(
        &self,
        event: ClientEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        Ok(())
    }
}
