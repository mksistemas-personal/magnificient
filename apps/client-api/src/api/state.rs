use crate::application::{
    ClientPersistence, CreateClientRequest, CreateClientService, DeleteClientRequest,
    DeleteClientService, FindClientsRequest, FindClientsService, GetClientByIdRequest,
    GetClientByIdService, UpdateClientRequest, UpdateClientService,
};
use crate::config::ClientAppConfig;
use crate::domain::Client;
use crate::infrastructure::PostgresClientRepository;
use database::DatabaseConnection;
use messaging::RabbitMqPublisher;
use shared::entities::BusinessError;
use shared::entities::address::repository::PostgresAddressRepository;
use shared::entities::use_case::use_case_traits::{
    AsyncAddEntityUseCase, AsyncDeleteEntityUseCase, AsyncGetAllUseCase, AsyncGetByIdUseCase,
    AsyncUpdateEntityUseCase,
};
use shared::types::Page;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ClientAppConfig>,
    pub tenant: String,
    pub db: DatabaseConnection,
    pub message_publisher: Option<Arc<RabbitMqPublisher>>,
    pub create_client: Arc<dyn AsyncAddEntityUseCase<CreateClientRequest, Client>>,
    pub update_client: Arc<dyn AsyncUpdateEntityUseCase<UpdateClientRequest, Client>>,
    pub delete_client: Arc<dyn AsyncDeleteEntityUseCase<DeleteClientRequest, ()>>,
    pub get_client: Arc<dyn AsyncGetByIdUseCase<GetClientByIdRequest, Client>>,
    pub find_clients: Arc<dyn AsyncGetAllUseCase<FindClientsRequest, Page<Client>>>,
}

impl AppState {
    pub fn new(config: Arc<ClientAppConfig>, db: DatabaseConnection) -> Self {
        let pool = db.pool().clone();
        let persistence = ClientPersistence::new(
            PostgresClientRepository::new(),
            PostgresAddressRepository::new(pool.clone()),
            pool,
        )
        .with_unit_of_work(db.unit_of_work());
        Self::from_persistence(config, db, String::new(), None, persistence)
    }

    pub fn new_with_publisher(
        config: Arc<ClientAppConfig>,
        db: DatabaseConnection,
        message_publisher: Arc<RabbitMqPublisher>,
    ) -> Self {
        let mut state = Self::new(config, db);
        state.message_publisher = Some(message_publisher);
        state
    }

    pub fn for_tenant_schema(&self, schema: &str, tenant: &str) -> Result<Self, BusinessError> {
        let unit_of_work = self
            .db
            .unit_of_work()
            .with_schema(schema.to_owned())
            .map_err(|error| BusinessError::invalid_data(error.to_string()))?;
        let pool = self.db.pool().clone();
        let persistence = ClientPersistence::new(
            PostgresClientRepository::new(),
            PostgresAddressRepository::new(pool.clone()),
            pool,
        )
        .with_unit_of_work(unit_of_work);
        Ok(Self::from_persistence(
            self.config.clone(),
            self.db.clone(),
            tenant.to_owned(),
            self.message_publisher.clone(),
            persistence,
        ))
    }

    fn from_persistence(
        config: Arc<ClientAppConfig>,
        db: DatabaseConnection,
        tenant: String,
        message_publisher: Option<Arc<RabbitMqPublisher>>,
        persistence: ClientPersistence,
    ) -> Self {
        Self {
            config,
            tenant,
            db,
            message_publisher,
            create_client: Arc::new(CreateClientService::new(persistence.clone())),
            update_client: Arc::new(UpdateClientService::new(persistence.clone())),
            delete_client: Arc::new(DeleteClientService::new(persistence.clone())),
            get_client: Arc::new(GetClientByIdService::new(persistence.clone())),
            find_clients: Arc::new(FindClientsService::new(persistence)),
        }
    }
}
