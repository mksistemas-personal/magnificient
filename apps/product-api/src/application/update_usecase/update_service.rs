use crate::application::ProductPersistence;
use crate::application::update_usecase::data::{InitialContextData, UpdateProductRequest};
use crate::application::update_usecase::states::Initial;
use crate::application::update_usecase::steps::{
    ChangeProductContext, LoadProductContext, PersistProductContext, ValidateInitialContext,
};
use crate::domain::Product;
use async_trait::async_trait;
use shared::entities::{AsyncUpdateEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct UpdateProductService {
    persistence: ProductPersistence,
}

impl UpdateProductService {
    pub fn new(persistence: ProductPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(&self, request: UpdateProductRequest) -> Result<Product, BusinessError> {
        <Self as AsyncUpdateEntityUseCase<UpdateProductRequest, Product>>::execute(self, request)
            .await
    }
}

#[async_trait]
impl AsyncUpdateEntityUseCase<UpdateProductRequest, Product> for UpdateProductService {
    async fn execute(&self, request: UpdateProductRequest) -> Result<Product, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .load(&self.persistence)
            .await?
            .change()?
            .save(&self.persistence)
            .await
    }
}
