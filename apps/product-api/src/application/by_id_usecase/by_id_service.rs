use crate::application::ProductPersistence;
use crate::application::by_id_usecase::data::{GetProductByIdRequest, InitialContextData};
use crate::application::by_id_usecase::states::Initial;
use crate::application::by_id_usecase::steps::{FindProductByIdContext, ValidateInitialContext};
use crate::domain::Product;
use async_trait::async_trait;
use shared::entities::{AsyncGetByIdUseCase, BusinessError, Context};
#[derive(Clone)]
pub struct GetProductByIdService {
    persistence: ProductPersistence,
}
impl GetProductByIdService {
    pub fn new(persistence: ProductPersistence) -> Self {
        Self { persistence }
    }
    pub async fn execute(&self, request: GetProductByIdRequest) -> Result<Product, BusinessError> {
        <Self as AsyncGetByIdUseCase<GetProductByIdRequest, Product>>::execute(self, request).await
    }
}
#[async_trait]
impl AsyncGetByIdUseCase<GetProductByIdRequest, Product> for GetProductByIdService {
    async fn execute(&self, request: GetProductByIdRequest) -> Result<Product, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .find(&self.persistence)
            .await
    }
}
