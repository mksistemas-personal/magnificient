use crate::application::SupplierPersistence;
use crate::application::delete_usecase::data::{DeleteSupplierRequest, InitialContextData};
use crate::application::delete_usecase::states::Initial;
use crate::application::delete_usecase::steps::{DeleteSupplierContext, ValidateInitialContext};
use async_trait::async_trait;
use shared::entities::{AsyncDeleteEntityUseCase, BusinessError, Context};
#[derive(Clone)]
pub struct DeleteSupplierService {
    persistence: SupplierPersistence,
}
impl DeleteSupplierService {
    pub fn new(persistence: SupplierPersistence) -> Self {
        Self { persistence }
    }
    pub async fn execute(&self, request: DeleteSupplierRequest) -> Result<(), BusinessError> {
        <Self as AsyncDeleteEntityUseCase<DeleteSupplierRequest, ()>>::execute(self, request).await
    }
}
#[async_trait]
impl AsyncDeleteEntityUseCase<DeleteSupplierRequest, ()> for DeleteSupplierService {
    async fn execute(&self, request: DeleteSupplierRequest) -> Result<(), BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .delete(&self.persistence)
            .await
    }
}
