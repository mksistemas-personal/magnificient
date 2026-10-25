use crate::application::addusecase::add_service::AddOrganizationService;
use crate::application::addusecase::data::AddOrganizationRequest;
use crate::application::allusecase::all_service::FindAllOrganizationsService;
use crate::application::allusecase::data::FindAllOrganizationsRequest;
use crate::application::updateusecase::UpdateOrganizationService;
use crate::application::updateusecase::data::UpdateOrganizationRequest;
use crate::application::{
    DeleteOrganizationRequest, DeleteOrganizationService, FindOrganizationByIdRequest,
    GetOrganizationByIdRequest, GetOrganizationByIdService,
};
use crate::config::organization_config::OrganizationAppConfig;
use crate::domain::organization::Organization;
use crate::domain::organization_repository::OrganizationRepository;
use crate::infrastructure::postgres_organization_repository::PostgresOrganizationRepository;
use database::DatabaseConnection;
use messaging::RabbitMqPublisher;
use shared::entities::AsyncAddEntityUseCase;
use shared::entities::use_case::use_case_traits::AsyncUpdateEntityUseCase;
use shared::entities::use_case::{
    AsyncDeleteEntityUseCase, AsyncGetAllUseCase, AsyncGetByIdUseCase,
};
use shared::types::page::Page;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<OrganizationAppConfig>,
    pub db: DatabaseConnection,
    pub message_publisher: Option<Arc<RabbitMqPublisher>>,
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
    pub fn new(config: Arc<OrganizationAppConfig>, db: DatabaseConnection) -> Self {
        let repository = PostgresOrganizationRepository::from_connection(&db);
        let uow = db.unit_of_work();
        Self {
            config,
            db,
            message_publisher: None,
            add_organization_use_case: Arc::new(AddOrganizationService::new_with_uow(
                repository.clone(),
                uow.clone(),
            )),
            update_organization_use_case: Arc::new(UpdateOrganizationService::new_with_uow(
                repository.clone(),
                uow.clone(),
            )),
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
            db,
            message_publisher: None,
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

    pub fn new_with_use_case(
        config: Arc<OrganizationAppConfig>,
        db: DatabaseConnection,
        add_use_case: Arc<dyn AsyncAddEntityUseCase<AddOrganizationRequest, Organization>>,
        update_use_case: Arc<dyn AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>>,
        delete_use_case: Arc<dyn AsyncDeleteEntityUseCase<DeleteOrganizationRequest, ()>>,
        get_by_id_use_case: Arc<dyn AsyncGetByIdUseCase<GetOrganizationByIdRequest, Organization>>,
        get_all_use_case: Arc<
            dyn AsyncGetAllUseCase<FindAllOrganizationsRequest, Page<Organization>>,
        >,
    ) -> Self {
        Self {
            config,
            db,
            message_publisher: None,
            add_organization_use_case: add_use_case,
            update_organization_use_case: update_use_case,
            delete_organization_use_case: delete_use_case,
            get_by_id_organization_use_case: get_by_id_use_case,
            get_all_organization_use_case: get_all_use_case,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::organization_config::{
        DatabaseConfig, LoggingConfig, OrganizationAppConfig, ServerConfig,
    };
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
        let db = DatabaseConnection::connect_lazy_with_config(&config.database.to_connection_config());
        let repo = InMemoryOrganizationRepository::new();
        let state = AppState::new_with_repository(Arc::clone(&config), db, repo);

        assert_eq!(state.config().server.port, 8080);
        assert!(state.message_publisher().is_none());
    }
}
