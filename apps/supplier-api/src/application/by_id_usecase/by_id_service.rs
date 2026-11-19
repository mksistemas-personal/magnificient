use crate::application::SupplierPersistence;
use crate::application::by_id_usecase::data::{GetSupplierByIdRequest, InitialContextData};
use crate::application::by_id_usecase::states::Initial;
use crate::application::by_id_usecase::steps::{FindSupplierByIdContext, ValidateInitialContext};
use crate::domain::Supplier;
use async_trait::async_trait;
use shared::entities::{AsyncGetByIdUseCase, BusinessError, Context};
#[derive(Clone)]
pub struct GetSupplierByIdService {
    persistence: SupplierPersistence,
}
impl GetSupplierByIdService {
    pub fn new(persistence: SupplierPersistence) -> Self {
        Self { persistence }
    }
    pub async fn execute(&self, request: GetSupplierByIdRequest) -> Result<Supplier, BusinessError> {
        <Self as AsyncGetByIdUseCase<GetSupplierByIdRequest, Supplier>>::execute(self, request).await
    }
}
#[async_trait]
impl AsyncGetByIdUseCase<GetSupplierByIdRequest, Supplier> for GetSupplierByIdService {
    async fn execute(&self, request: GetSupplierByIdRequest) -> Result<Supplier, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .find(&self.persistence)
            .await
    }
}
