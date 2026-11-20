use crate::application::ProductPersistence;
use crate::application::delete_usecase::data::{DeleteProductRequest, InitialContextData};
use crate::application::delete_usecase::states::Initial;
use crate::application::delete_usecase::steps::{DeleteProductContext, ValidateInitialContext};
use async_trait::async_trait;
use shared::entities::{AsyncDeleteEntityUseCase, BusinessError, Context};
#[derive(Clone)]
pub struct DeleteProductService {
    persistence: ProductPersistence,
}
impl DeleteProductService {
    pub fn new(persistence: ProductPersistence) -> Self {
        Self { persistence }
    }
    pub async fn execute(&self, request: DeleteProductRequest) -> Result<(), BusinessError> {
        <Self as AsyncDeleteEntityUseCase<DeleteProductRequest, ()>>::execute(self, request).await
    }
}
#[async_trait]
impl AsyncDeleteEntityUseCase<DeleteProductRequest, ()> for DeleteProductService {
    async fn execute(&self, request: DeleteProductRequest) -> Result<(), BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .delete(&self.persistence)
            .await
    }
}
