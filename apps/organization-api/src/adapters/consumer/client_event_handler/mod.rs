pub mod data;
pub mod states;
pub mod steps;

use crate::adapters::api::AppState;
use crate::adapters::consumer::client_event_handler::states::Initial;
use async_trait::async_trait;
use client_contracts::v1::{ClientEvent, ClientEventType};
use messaging::{MessageHandler, MessageHandlerError, MessageMetadata};
use shared::entities::Context;

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

    async fn handle_delete_client(
        &self,
        event: ClientEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        use steps::{
            ParseClientAndOrganizationId, RemoveClientFromOrganization, ResolveTenant,
            ValidateDeleteEvent,
        };

        Context::<Initial, _>::new(data::InitialContextData { event, metadata })
            .validate_delete_event()?
            .resolve_tenant(&self.app_state)
            .await?
            .parse_client_and_organization_id()?
            .unlink_client_organization()
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
        match event.event_type {
            ClientEventType::AddClient => {
                self.handle_add_client(event, metadata).await?;
            }
            ClientEventType::DeleteClient => {
                self.handle_delete_client(event, metadata).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::organization_config::{
        DatabaseConfig, LoggingConfig, OrganizationAppConfig, ServerConfig,
    };
    use crate::domain::{ClientId, Organization, OrganizationEntityType, OrganizationId, OrganizationLink};
    use crate::infrastructure::in_memory_organization_repository::InMemoryOrganizationRepository;
    use messaging::RabbitMqConfig;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};
    use std::sync::Arc;
    use steps::{
        AddClientToOrganization, ParseClientAndOrganizationId, RemoveClientFromOrganization,
        ValidateDeleteEvent, ValidateEvent,
    };

    fn create_test_app_state() -> (AppState, InMemoryOrganizationRepository) {
        let config = Arc::new(OrganizationAppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                name: "test".to_string(),
                username: "postgres".to_string(),
                password: "password".to_string(),
                max_connections: None,
                min_connections: None,
                acquire_timeout_secs: None,
            },
            rabbitmq: RabbitMqConfig {
                rabbitmq_url: "amqp://guest:guest@localhost:5672".to_string(),
                rabbitmq_prefetch_count: 10,
                rabbitmq_reconnect_max_attempts: 3,
                rabbitmq_reconnect_initial_ms: 100,
                rabbitmq_reconnect_max_ms: 1000,
                rabbitmq_retry_max_attempts: 3,
                rabbitmq_retry_initial_ms: 100,
                rabbitmq_retry_max_ms: 1000,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
        });
        let db = database::DatabaseConnection::connect_lazy_with_config(
            &config.database.to_connection_config(),
        );
        let repo = InMemoryOrganizationRepository::new();
        let state = AppState::new_with_repository(config, db, repo.clone());
        (state, repo)
    }

    #[test]
    fn test_validate_delete_event_success() {
        let event = ClientEvent::delete("123456789");
        let metadata = MessageMetadata {
            event_type: "ClientEvent".to_string(),
            version: 100,
            message_id: Some("msg-1".to_string()),
            source: None,
            tenant: Some("tenant-a".to_string()),
        };

        let context = Context::<Initial, _>::new(data::InitialContextData { event, metadata });
        let validated = context.validate_delete_event();

        assert!(validated.is_ok());
        let data = validated.unwrap().data;
        assert_eq!(data.client_id, "123456789");
        assert_eq!(data.tenant, "tenant-a");
    }

    #[test]
    fn test_validate_delete_event_wrong_type() {
        let event = ClientEvent::add("123456789", "987654321", "Client A", None, None);
        let metadata = MessageMetadata {
            event_type: "ClientEvent".to_string(),
            version: 100,
            message_id: Some("msg-1".to_string()),
            source: None,
            tenant: Some("tenant-a".to_string()),
        };

        let context = Context::<Initial, _>::new(data::InitialContextData { event, metadata });
        let result = context.validate_delete_event();

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_delete_event_missing_tenant() {
        let event = ClientEvent::delete("123456789");
        let metadata = MessageMetadata {
            event_type: "ClientEvent".to_string(),
            version: 100,
            message_id: Some("msg-1".to_string()),
            source: None,
            tenant: None,
        };

        let context = Context::<Initial, _>::new(data::InitialContextData { event, metadata });
        let result = context.validate_delete_event();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unlink_client_organization_step_removes_link() {
        let (state, repo) = create_test_app_state();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let client_id = ClientId::new(TSID::from(200u64));

        let org = Organization::new(
            org_id,
            "Empresa Teste",
            Document::new("00.000.000/0001-91", DocumentType::Cnpj),
        )
        .with_audit(AuditFields::created("admin"));
        repo.insert_for_test(org);

        let link = OrganizationLink::new(
            org_id,
            *client_id,
            OrganizationEntityType::Client,
            "client-consumer",
        );
        repo.insert_link_for_test(link);
        assert_eq!(repo.links().len(), 1);

        let context = Context::new(data::ClientIdParsedContextData {
            client_id,
            organization_id: None,
            tenant_state: state,
        });

        let result = context.unlink_client_organization().await;
        assert!(result.is_ok());
        assert_eq!(repo.links().len(), 0);
    }

    #[test]
    fn test_validate_add_event_success() {
        let event = ClientEvent::add("123456789", "987654321", "Client A", None, None);
        let metadata = MessageMetadata {
            event_type: "ClientEvent".to_string(),
            version: 100,
            message_id: Some("msg-1".to_string()),
            source: None,
            tenant: Some("tenant-a".to_string()),
        };

        let context = Context::<Initial, _>::new(data::InitialContextData { event, metadata });
        let validated = context.validate_event();

        assert!(validated.is_ok());
        let data = validated.unwrap().data;
        assert_eq!(data.client_id, "123456789");
        assert_eq!(data.organization_id, Some("987654321".to_string()));
        assert_eq!(data.tenant, "tenant-a");
    }

    #[tokio::test]
    async fn test_link_client_organization_step_creates_link() {
        let (state, repo) = create_test_app_state();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let client_id = ClientId::new(TSID::from(200u64));

        let org = Organization::new(
            org_id,
            "Empresa Teste",
            Document::new("00.000.000/0001-91", DocumentType::Cnpj),
        )
        .with_audit(AuditFields::created("admin"));
        repo.insert_for_test(org);
        assert_eq!(repo.links().len(), 0);

        let context = Context::new(data::ClientIdParsedContextData {
            client_id,
            organization_id: Some(org_id),
            tenant_state: state,
        });

        let result = context.link_client_organization().await;
        assert!(result.is_ok());
        assert_eq!(repo.links().len(), 1);
        assert_eq!(repo.links()[0].organization_id(), org_id);
        assert_eq!(repo.links()[0].entity_id(), *client_id);
    }

    #[tokio::test]
    async fn test_parse_ids_step() {
        let (state, _) = create_test_app_state();
        let context = Context::new(data::TenantResolvedContextData {
            client_id: "200".to_string(),
            organization_id: Some("100".to_string()),
            tenant_state: state.clone(),
        });

        let parsed = context.parse_client_and_organization_id();
        assert!(parsed.is_ok());
        let data = parsed.unwrap().data;
        assert_eq!(*data.client_id, TSID::from(200u64));
        assert_eq!(
            data.organization_id.unwrap(),
            OrganizationId::new(TSID::from(100u64))
        );

        let context_delete = Context::new(data::TenantResolvedContextData {
            client_id: "200".to_string(),
            organization_id: None,
            tenant_state: state,
        });

        let parsed_delete = context_delete.parse_client_and_organization_id();
        assert!(parsed_delete.is_ok());
        let data_delete = parsed_delete.unwrap().data;
        assert_eq!(*data_delete.client_id, TSID::from(200u64));
        assert!(data_delete.organization_id.is_none());
    }
}
