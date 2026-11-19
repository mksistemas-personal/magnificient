pub mod data;
pub mod states;
pub mod steps;

use crate::adapters::api::AppState;
use crate::adapters::consumer::supplier_event_handler::states::Initial;
use async_trait::async_trait;
use messaging::{MessageHandler, MessageHandlerError, MessageMetadata};
use shared::entities::Context;
use supplier_contracts::v1::{SupplierEvent, SupplierEventType};

pub struct SupplierEventHandler {
    app_state: AppState,
}

impl SupplierEventHandler {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn handle_add_supplier(
        &self,
        event: SupplierEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        use steps::{AddSupplierToOrganization, ParseSupplierAndOrganizationId, ResolveTenant, ValidateEvent};

        Context::<Initial, _>::new(data::InitialContextData { event, metadata })
            .validate_event()?
            .resolve_tenant(&self.app_state)
            .await?
            .parse_supplier_and_organization_id()?
            .link_supplier_organization()
            .await
    }

    async fn handle_delete_supplier(
        &self,
        event: SupplierEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        use steps::{
            ParseSupplierAndOrganizationId, RemoveSupplierFromOrganization, ResolveTenant,
            ValidateDeleteEvent,
        };

        Context::<Initial, _>::new(data::InitialContextData { event, metadata })
            .validate_delete_event()?
            .resolve_tenant(&self.app_state)
            .await?
            .parse_supplier_and_organization_id()?
            .unlink_supplier_organization()
            .await
    }
}

#[async_trait]
impl MessageHandler<SupplierEvent> for SupplierEventHandler {
    async fn handle(
        &self,
        event: SupplierEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        match event.event_type {
            SupplierEventType::AddSupplier => {
                self.handle_add_supplier(event, metadata).await?;
            }
            SupplierEventType::DeleteSupplier => {
                self.handle_delete_supplier(event, metadata).await?;
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
    use crate::domain::{Organization, OrganizationEntityType, OrganizationId, OrganizationLink, SupplierId};
    use crate::infrastructure::in_memory_organization_repository::InMemoryOrganizationRepository;
    use messaging::RabbitMqConfig;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};
    use std::sync::Arc;
    use steps::{
        AddSupplierToOrganization, ParseSupplierAndOrganizationId, RemoveSupplierFromOrganization,
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
        let event = SupplierEvent::delete("123456789");
        let metadata = MessageMetadata {
            event_type: "SupplierEvent".to_string(),
            version: 100,
            message_id: Some("msg-1".to_string()),
            source: None,
            tenant: Some("tenant-a".to_string()),
        };

        let context = Context::<Initial, _>::new(data::InitialContextData { event, metadata });
        let validated = context.validate_delete_event();

        assert!(validated.is_ok());
        let data = validated.unwrap().data;
        assert_eq!(data.supplier_id, "123456789");
        assert_eq!(data.tenant, "tenant-a");
    }

    #[test]
    fn test_validate_delete_event_wrong_type() {
        let event = SupplierEvent::add("123456789", "987654321", "Supplier A", None, None);
        let metadata = MessageMetadata {
            event_type: "SupplierEvent".to_string(),
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
        let event = SupplierEvent::delete("123456789");
        let metadata = MessageMetadata {
            event_type: "SupplierEvent".to_string(),
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
    async fn test_unlink_supplier_organization_step_removes_link() {
        let (state, repo) = create_test_app_state();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let supplier_id = SupplierId::new(TSID::from(200u64));

        let org = Organization::new(
            org_id,
            "Empresa Teste",
            Document::new("00.000.000/0001-91", DocumentType::Cnpj),
        )
        .with_audit(AuditFields::created("admin"));
        repo.insert_for_test(org);

        let link = OrganizationLink::new(
            org_id,
            *supplier_id,
            OrganizationEntityType::Supplier,
            "supplier-consumer",
        );
        repo.insert_link_for_test(link);
        assert_eq!(repo.links().len(), 1);

        let context = Context::new(data::SupplierIdParsedContextData {
            supplier_id,
            organization_id: None,
            tenant_state: state,
        });

        let result = context.unlink_supplier_organization().await;
        assert!(result.is_ok());
        assert_eq!(repo.links().len(), 0);
    }

    #[test]
    fn test_validate_add_event_success() {
        let event = SupplierEvent::add("123456789", "987654321", "Supplier A", None, None);
        let metadata = MessageMetadata {
            event_type: "SupplierEvent".to_string(),
            version: 100,
            message_id: Some("msg-1".to_string()),
            source: None,
            tenant: Some("tenant-a".to_string()),
        };

        let context = Context::<Initial, _>::new(data::InitialContextData { event, metadata });
        let validated = context.validate_event();

        assert!(validated.is_ok());
        let data = validated.unwrap().data;
        assert_eq!(data.supplier_id, "123456789");
        assert_eq!(data.organization_id, Some("987654321".to_string()));
        assert_eq!(data.tenant, "tenant-a");
    }

    #[tokio::test]
    async fn test_link_supplier_organization_step_creates_link() {
        let (state, repo) = create_test_app_state();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let supplier_id = SupplierId::new(TSID::from(200u64));

        let org = Organization::new(
            org_id,
            "Empresa Teste",
            Document::new("00.000.000/0001-91", DocumentType::Cnpj),
        )
        .with_audit(AuditFields::created("admin"));
        repo.insert_for_test(org);
        assert_eq!(repo.links().len(), 0);

        let context = Context::new(data::SupplierIdParsedContextData {
            supplier_id,
            organization_id: Some(org_id),
            tenant_state: state,
        });

        let result = context.link_supplier_organization().await;
        assert!(result.is_ok());
        assert_eq!(repo.links().len(), 1);
        assert_eq!(repo.links()[0].organization_id(), org_id);
        assert_eq!(repo.links()[0].entity_id(), *supplier_id);
    }

    #[tokio::test]
    async fn test_parse_ids_step() {
        let (state, _) = create_test_app_state();
        let context = Context::new(data::TenantResolvedContextData {
            supplier_id: "200".to_string(),
            organization_id: Some("100".to_string()),
            tenant_state: state.clone(),
        });

        let parsed = context.parse_supplier_and_organization_id();
        assert!(parsed.is_ok());
        let data = parsed.unwrap().data;
        assert_eq!(*data.supplier_id, TSID::from(200u64));
        assert_eq!(
            data.organization_id.unwrap(),
            OrganizationId::new(TSID::from(100u64))
        );

        let context_delete = Context::new(data::TenantResolvedContextData {
            supplier_id: "200".to_string(),
            organization_id: None,
            tenant_state: state,
        });

        let parsed_delete = context_delete.parse_supplier_and_organization_id();
        assert!(parsed_delete.is_ok());
        let data_delete = parsed_delete.unwrap().data;
        assert_eq!(*data_delete.supplier_id, TSID::from(200u64));
        assert!(data_delete.organization_id.is_none());
    }
}
