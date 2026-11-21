use crate::application::{
    ProductPersistence, CreateProductRequest, CreateProductService, DeleteProductRequest,
    DeleteProductService, DeleteOrganizationProductsService, FindProductsRequest, FindProductsService,
    GetProductByIdRequest, GetProductByIdService, UpdateProductRequest, UpdateProductService,
};
use crate::config::ProductAppConfig;
use crate::domain::Product;
use crate::infrastructure::PostgresProductRepository;
use database::DatabaseConnection;
use messaging::RabbitMqPublisher;
use shared::entities::{
    AsyncAddEntityUseCase, AsyncDeleteEntityUseCase, AsyncGetAllUseCase, AsyncGetByIdUseCase,
    AsyncUpdateEntityUseCase, BusinessError,
};
use shared::types::Page;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ProductAppConfig>,
    pub tenant: String,
    pub db: DatabaseConnection,
    pub message_publisher: Option<Arc<RabbitMqPublisher>>,
    pub create_product: Arc<dyn AsyncAddEntityUseCase<CreateProductRequest, Product>>,
    pub update_product: Arc<dyn AsyncUpdateEntityUseCase<UpdateProductRequest, Product>>,
    pub delete_product: Arc<dyn AsyncDeleteEntityUseCase<DeleteProductRequest, ()>>,
    pub delete_organization_products: Arc<DeleteOrganizationProductsService>,
    pub get_product: Arc<dyn AsyncGetByIdUseCase<GetProductByIdRequest, Product>>,
    pub find_products: Arc<dyn AsyncGetAllUseCase<FindProductsRequest, Page<Product>>>,
}

impl AppState {
    pub fn new(config: Arc<ProductAppConfig>, db: DatabaseConnection) -> Self {
        let pool = db.pool().clone();
        let persistence = ProductPersistence::new(
            PostgresProductRepository::new(),
            pool,
        )
        .with_unit_of_work(db.unit_of_work());
        Self::from_persistence(config, db, String::new(), None, persistence)
    }

    pub fn new_with_publisher(
        config: Arc<ProductAppConfig>,
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
        let persistence = ProductPersistence::new(
            PostgresProductRepository::new(),
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
        config: Arc<ProductAppConfig>,
        db: DatabaseConnection,
        tenant: String,
        message_publisher: Option<Arc<RabbitMqPublisher>>,
        persistence: ProductPersistence,
    ) -> Self {
        Self {
            config,
            tenant,
            db,
            message_publisher,
            create_product: Arc::new(CreateProductService::new(persistence.clone())),
            update_product: Arc::new(UpdateProductService::new(persistence.clone())),
            delete_product: Arc::new(DeleteProductService::new(persistence.clone())),
            delete_organization_products: Arc::new(DeleteOrganizationProductsService::new(
                persistence.clone(),
            )),
            get_product: Arc::new(GetProductByIdService::new(persistence.clone())),
            find_products: Arc::new(FindProductsService::new(persistence)),
        }
    }
}
