use crate::application::{
    AddOrganizationRequest, AddOrganizationService, DeleteOrganizationRequest,
    DeleteOrganizationService, FindAllOrganizationsRequest, FindAllOrganizationsService,
    FindOrganizationByIdRequest, GetOrganizationByIdService, UpdateOrganizationRequest,
    UpdateOrganizationService,
};
use crate::config::organization_config::OrganizationAppConfig;
use crate::domain::organization::Organization;
use crate::domain::organization_repository::OrganizationRepository;
use crate::infrastructure::PostgresAddressRepository;
use crate::infrastructure::postgres_organization_repository::PostgresOrganizationRepository;
use database::DatabaseConnection;
use messaging::RabbitMqPublisher;
use shared::entities::use_case::use_case_traits::AsyncUpdateEntityUseCase;
use shared::entities::use_case::{
    AsyncDeleteEntityUseCase, AsyncGetAllUseCase, AsyncGetByIdUseCase,
};
use shared::entities::{AddressRepository, AsyncAddEntityUseCase};
use shared::types::page::Page;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<OrganizationAppConfig>,
    pub tenant: String,
    pub db: DatabaseConnection,
    pub message_publisher: Option<Arc<RabbitMqPublisher>>,
    pub address_repository: Option<Arc<dyn AddressRepository>>,
    pub add_organization_use_case:
        Arc<dyn AsyncAddEntityUseCase<AddOrganizationRequest, Organization>>,
    pub update_organization_use_case:
        Arc<dyn AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>>,
    pub delete_organization_use_case:
        Arc<dyn AsyncDeleteEntityUseCase<DeleteOrganizationRequest, ()>>,
    pub get_by_id_organization_use_case:
        Arc<dyn AsyncGetByIdUseCase<FindOrganizationByIdRequest, Organization>>,
    pub get_all_organization_use_case:
        Arc<dyn AsyncGetAllUseCase<FindAllOrganizationsRequest, Page<Organization>>>,
}

impl AppState {
    /// Creates request-scoped repositories and a UnitOfWork pinned to a tenant schema.
    /// Schema names must come from a trusted tenant registry, not directly from user input.
    pub fn for_tenant_schema(
        &self,
        schema: &str,
        tenant: &str,
    ) -> Result<Self, shared::entities::BusinessError> {
        let uow = self
            .db
            .unit_of_work()
            .with_schema(schema.to_owned())
            .map_err(|e| shared::entities::BusinessError::invalid_data(e.to_string()))?;
        let repository = PostgresOrganizationRepository::new(self.db.pool().clone());
        let address_repository = PostgresAddressRepository::new(self.db.pool().clone());
        Ok(Self {
            config: self.config.clone(),
            tenant: tenant.to_owned(),
            db: self.db.clone(),
            message_publisher: self.message_publisher.clone(),
            address_repository: Some(Arc::new(address_repository.clone())),
            add_organization_use_case: Arc::new(
                AddOrganizationService::new_with_repositories_and_uow(
                    repository.clone(),
                    address_repository.clone(),
                    uow.clone(),
                ),
            ),
            update_organization_use_case: Arc::new(
                UpdateOrganizationService::new_with_repositories_and_uow(
                    repository.clone(),
                    address_repository,
                    uow.clone(),
                ),
            ),
            delete_organization_use_case: Arc::new(DeleteOrganizationService::new_with_uow(
                repository.clone(),
                uow.clone(),
            )),
            get_by_id_organization_use_case: Arc::new(GetOrganizationByIdService::new_with_uow(
                repository.clone(),
                uow.clone(),
            )),
            get_all_organization_use_case: Arc::new(FindAllOrganizationsService::new_with_uow(
                repository, uow,
            )),
        })
    }

    pub fn new(config: Arc<OrganizationAppConfig>, db: DatabaseConnection) -> Self {
        let repository = PostgresOrganizationRepository::new(db.pool().clone());
        let address_repository = PostgresAddressRepository::new(db.pool().clone());
        let uow = db.unit_of_work();
        Self {
            config,
            tenant: String::new(),
            db,
            message_publisher: None,
            address_repository: Some(Arc::new(address_repository.clone())),
            add_organization_use_case: Arc::new(
                AddOrganizationService::new_with_repositories_and_uow(
                    repository.clone(),
                    address_repository.clone(),
                    uow.clone(),
                ),
            ),
            update_organization_use_case: Arc::new(
                UpdateOrganizationService::new_with_repositories_and_uow(
                    repository.clone(),
                    address_repository,
                    uow.clone(),
                ),
            ),
            delete_organization_use_case: Arc::new(DeleteOrganizationService::new_with_uow(
                repository.clone(),
                uow.clone(),
            )),
            get_by_id_organization_use_case: Arc::new(GetOrganizationByIdService::new_with_uow(
                repository.clone(),
                uow.clone(),
            )),
            get_all_organization_use_case: Arc::new(FindAllOrganizationsService::new_with_uow(
                repository, uow,
            )),
        }
    }

    pub fn new_with_publisher(
        config: Arc<OrganizationAppConfig>,
        db: DatabaseConnection,
        message_publisher: Arc<RabbitMqPublisher>,
    ) -> Self {
        let mut state = Self::new(config, db);
        state.message_publisher = Some(message_publisher);
        state
    }

    pub fn with_message_publisher(mut self, message_publisher: Arc<RabbitMqPublisher>) -> Self {
        self.message_publisher = Some(message_publisher);
        self
    }

    pub fn new_with_repository(
        config: Arc<OrganizationAppConfig>,
        db: DatabaseConnection,
        repository: impl OrganizationRepository + Clone + 'static,
    ) -> Self {
        Self {
            config,
            tenant: String::new(),
            db,
            message_publisher: None,
            address_repository: None,
            add_organization_use_case: Arc::new(AddOrganizationService::new(repository.clone())),
            update_organization_use_case: Arc::new(UpdateOrganizationService::new(
                repository.clone(),
            )),
            delete_organization_use_case: Arc::new(DeleteOrganizationService::new(
                repository.clone(),
            )),
            get_by_id_organization_use_case: Arc::new(GetOrganizationByIdService::new(
                repository.clone(),
            )),
            get_all_organization_use_case: Arc::new(FindAllOrganizationsService::new(repository)),
        }
    }

    pub fn new_with_repositories(
        config: Arc<OrganizationAppConfig>,
        db: DatabaseConnection,
        repository: impl OrganizationRepository + Clone + 'static,
        address_repository: impl AddressRepository + Clone + 'static,
    ) -> Self {
        Self {
            config,
            tenant: String::new(),
            db,
            message_publisher: None,
            address_repository: Some(Arc::new(address_repository.clone())),
            add_organization_use_case: Arc::new(AddOrganizationService::new_with_repositories(
                repository.clone(),
                address_repository.clone(),
            )),
            update_organization_use_case: Arc::new(
                UpdateOrganizationService::new_with_repositories(
                    repository.clone(),
                    address_repository,
                ),
            ),
            delete_organization_use_case: Arc::new(DeleteOrganizationService::new(
                repository.clone(),
            )),
            get_by_id_organization_use_case: Arc::new(GetOrganizationByIdService::new(
                repository.clone(),
            )),
            get_all_organization_use_case: Arc::new(FindAllOrganizationsService::new(repository)),
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn config(&self) -> &OrganizationAppConfig {
        &self.config
    }

    pub fn message_publisher(&self) -> Option<&Arc<RabbitMqPublisher>> {
        self.message_publisher.as_ref()
    }

    pub fn address_repository(&self) -> Option<&Arc<dyn AddressRepository>> {
        self.address_repository.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::organization_config::{
        DatabaseConfig, LoggingConfig, OrganizationAppConfig, ServerConfig,
    };
    use crate::infrastructure::InMemoryAddressRepository;
    use crate::infrastructure::in_memory_organization_repository::InMemoryOrganizationRepository;
    use messaging::RabbitMqConfig;

    fn create_test_config() -> Arc<OrganizationAppConfig> {
        Arc::new(OrganizationAppConfig {
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
        })
    }

    #[tokio::test]
    async fn test_app_state_initialization() {
        let config = create_test_config();
        let db =
            DatabaseConnection::connect_lazy_with_config(&config.database.to_connection_config());
        let repo = InMemoryOrganizationRepository::new();
        let state = AppState::new_with_repository(Arc::clone(&config), db, repo);

        assert_eq!(state.config().server.port, 8080);
        assert!(state.tenant.is_empty());
        assert!(state.message_publisher().is_none());
    }

    #[tokio::test]
    async fn test_app_state_for_tenant_schema_keeps_tenant_slug() {
        let config = create_test_config();
        let db =
            DatabaseConnection::connect_lazy_with_config(&config.database.to_connection_config());
        let state = AppState::new(Arc::clone(&config), db);

        let tenant_state = state
            .for_tenant_schema("tenant_schema", "tenant-slug")
            .unwrap();

        assert_eq!(tenant_state.tenant, "tenant-slug");
    }

    #[tokio::test]
    async fn test_app_state_with_repositories() {
        let config = create_test_config();
        let db =
            DatabaseConnection::connect_lazy_with_config(&config.database.to_connection_config());
        let repo = InMemoryOrganizationRepository::new();
        let addr_repo = InMemoryAddressRepository::new();
        let state = AppState::new_with_repositories(Arc::clone(&config), db, repo, addr_repo);

        assert_eq!(state.config().server.port, 8080);
        assert!(state.address_repository().is_some());
    }
}
