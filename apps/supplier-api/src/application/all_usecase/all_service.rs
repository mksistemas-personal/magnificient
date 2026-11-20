use crate::application::SupplierPersistence;
use crate::application::all_usecase::data::{FindSuppliersRequest, InitialContextData};
use crate::application::all_usecase::states::Initial;
use crate::application::all_usecase::steps::{FindSuppliersContext, ValidateInitialContext};
use crate::domain::Supplier;
use async_trait::async_trait;
use shared::entities::{AsyncGetAllUseCase, BusinessError, Context};
use shared::types::Page;
#[derive(Clone)]
pub struct FindSuppliersService {
    persistence: SupplierPersistence,
}
impl FindSuppliersService {
    pub fn new(persistence: SupplierPersistence) -> Self {
        Self { persistence }
    }
    pub async fn execute(
        &self,
        request: FindSuppliersRequest,
    ) -> Result<Page<Supplier>, BusinessError> {
        <Self as AsyncGetAllUseCase<FindSuppliersRequest, Page<Supplier>>>::execute(self, request).await
    }
}
#[async_trait]
impl AsyncGetAllUseCase<FindSuppliersRequest, Page<Supplier>> for FindSuppliersService {
    async fn execute(&self, request: FindSuppliersRequest) -> Result<Page<Supplier>, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .find(&self.persistence)
            .await
    }
}
