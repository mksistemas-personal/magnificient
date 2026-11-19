use crate::application::SupplierPersistence;
use crate::application::add_usecase::data::{CreateSupplierRequest, InitialContextData};
use crate::application::add_usecase::states::Initial;
use crate::application::add_usecase::steps::{
    BuildSupplierContext, PersistSupplierContext, ValidateAddressContext, ValidateInitialContext,
};
use crate::domain::Supplier;
use async_trait::async_trait;
use shared::entities::{AsyncAddEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct CreateSupplierService {
    persistence: SupplierPersistence,
}

impl CreateSupplierService {
    pub fn new(persistence: SupplierPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(&self, request: CreateSupplierRequest) -> Result<Supplier, BusinessError> {
        <Self as AsyncAddEntityUseCase<CreateSupplierRequest, Supplier>>::execute(self, request).await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<CreateSupplierRequest, Supplier> for CreateSupplierService {
    async fn execute(&self, request: CreateSupplierRequest) -> Result<Supplier, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .validate_address()?
            .build()?
            .save(&self.persistence)
            .await
    }
}
