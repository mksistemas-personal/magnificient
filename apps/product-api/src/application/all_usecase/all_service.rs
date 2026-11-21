use crate::application::ProductPersistence;
use crate::application::all_usecase::data::{FindProductsRequest, InitialContextData};
use crate::application::all_usecase::states::Initial;
use crate::application::all_usecase::steps::{FindProductsContext, ValidateInitialContext};
use crate::domain::Product;
use async_trait::async_trait;
use shared::entities::{AsyncGetAllUseCase, BusinessError, Context};
use shared::types::Page;
#[derive(Clone)]
pub struct FindProductsService {
    persistence: ProductPersistence,
}
impl FindProductsService {
    pub fn new(persistence: ProductPersistence) -> Self {
        Self { persistence }
    }
    pub async fn execute(
        &self,
        request: FindProductsRequest,
    ) -> Result<Page<Product>, BusinessError> {
        <Self as AsyncGetAllUseCase<FindProductsRequest, Page<Product>>>::execute(self, request).await
    }
}
#[async_trait]
impl AsyncGetAllUseCase<FindProductsRequest, Page<Product>> for FindProductsService {
    async fn execute(&self, request: FindProductsRequest) -> Result<Page<Product>, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .find(&self.persistence)
            .await
    }
}
