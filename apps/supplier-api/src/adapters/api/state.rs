use crate::application::{
    SupplierPersistence, CreateSupplierRequest, CreateSupplierService, DeleteSupplierRequest,
    DeleteSupplierService, DeleteOrganizationSuppliersService, FindSuppliersRequest, FindSuppliersService,
    GetSupplierByIdRequest, GetSupplierByIdService, UpdateSupplierRequest, UpdateSupplierService,
};
use crate::config::SupplierAppConfig;
use crate::domain::Supplier;
use crate::infrastructure::PostgresSupplierRepository;
use database::DatabaseConnection;
use messaging::RabbitMqPublisher;
use shared::entities::{
    AsyncAddEntityUseCase, AsyncDeleteEntityUseCase, AsyncGetAllUseCase, AsyncGetByIdUseCase,
    AsyncUpdateEntityUseCase, BusinessError, PostgresAddressRepository,
};
use shared::types::Page;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<SupplierAppConfig>,
    pub tenant: String,
    pub db: DatabaseConnection,
    pub message_publisher: Option<Arc<RabbitMqPublisher>>,
    pub create_supplier: Arc<dyn AsyncAddEntityUseCase<CreateSupplierRequest, Supplier>>,
    pub update_supplier: Arc<dyn AsyncUpdateEntityUseCase<UpdateSupplierRequest, Supplier>>,
    pub delete_supplier: Arc<dyn AsyncDeleteEntityUseCase<DeleteSupplierRequest, ()>>,
    pub delete_organization_suppliers: Arc<DeleteOrganizationSuppliersService>,
    pub get_supplier: Arc<dyn AsyncGetByIdUseCase<GetSupplierByIdRequest, Supplier>>,
    pub find_suppliers: Arc<dyn AsyncGetAllUseCase<FindSuppliersRequest, Page<Supplier>>>,
}

impl AppState {
    pub fn new(config: Arc<SupplierAppConfig>, db: DatabaseConnection) -> Self {
        let pool = db.pool().clone();
        let persistence = SupplierPersistence::new(
            PostgresSupplierRepository::new(),
            PostgresAddressRepository::new(pool.clone()),
            pool,
        )
        .with_unit_of_work(db.unit_of_work());
        Self::from_persistence(config, db, String::new(), None, persistence)
    }

    pub fn new_with_publisher(
        config: Arc<SupplierAppConfig>,
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
        let persistence = SupplierPersistence::new(
            PostgresSupplierRepository::new(),
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
        config: Arc<SupplierAppConfig>,
        db: DatabaseConnection,
        tenant: String,
        message_publisher: Option<Arc<RabbitMqPublisher>>,
        persistence: SupplierPersistence,
    ) -> Self {
        Self {
            config,
            tenant,
            db,
            message_publisher,
            create_supplier: Arc::new(CreateSupplierService::new(persistence.clone())),
            update_supplier: Arc::new(UpdateSupplierService::new(persistence.clone())),
            delete_supplier: Arc::new(DeleteSupplierService::new(persistence.clone())),
            delete_organization_suppliers: Arc::new(DeleteOrganizationSuppliersService::new(
                persistence.clone(),
            )),
            get_supplier: Arc::new(GetSupplierByIdService::new(persistence.clone())),
            find_suppliers: Arc::new(FindSuppliersService::new(persistence)),
        }
    }
}
