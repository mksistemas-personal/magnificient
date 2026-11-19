use crate::application::SupplierPersistence;
use crate::application::update_usecase::data::{InitialContextData, UpdateSupplierRequest};
use crate::application::update_usecase::states::Initial;
use crate::application::update_usecase::steps::{
    ChangeSupplierContext, LoadSupplierContext, PersistSupplierContext, ValidateAddressContext,
    ValidateInitialContext,
};
use crate::domain::Supplier;
use async_trait::async_trait;
use shared::entities::{AsyncUpdateEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct UpdateSupplierService {
    persistence: SupplierPersistence,
}

impl UpdateSupplierService {
    pub fn new(persistence: SupplierPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(&self, request: UpdateSupplierRequest) -> Result<Supplier, BusinessError> {
        <Self as AsyncUpdateEntityUseCase<UpdateSupplierRequest, Supplier>>::execute(self, request)
            .await
    }
}

#[async_trait]
impl AsyncUpdateEntityUseCase<UpdateSupplierRequest, Supplier> for UpdateSupplierService {
    async fn execute(&self, request: UpdateSupplierRequest) -> Result<Supplier, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .validate_address()?
            .load(&self.persistence)
            .await?
            .change()?
            .save(&self.persistence)
            .await
    }
}
