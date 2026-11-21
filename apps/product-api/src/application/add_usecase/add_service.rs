use crate::application::ProductPersistence;
use crate::application::add_usecase::data::{CreateProductRequest, InitialContextData};
use crate::application::add_usecase::states::Initial;
use crate::application::add_usecase::steps::{
    BuildProductContext, PersistProductContext, ValidateInitialContext,
};
use crate::domain::Product;
use async_trait::async_trait;
use shared::entities::{AsyncAddEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct CreateProductService {
    persistence: ProductPersistence,
}

impl CreateProductService {
    pub fn new(persistence: ProductPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(&self, request: CreateProductRequest) -> Result<Product, BusinessError> {
        <Self as AsyncAddEntityUseCase<CreateProductRequest, Product>>::execute(self, request).await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<CreateProductRequest, Product> for CreateProductService {
    async fn execute(&self, request: CreateProductRequest) -> Result<Product, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .build()?
            .save(&self.persistence)
            .await
    }
}
