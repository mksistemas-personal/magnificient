use crate::application::delete_usecase::data::{InitialContextData, ValidatedContextData};
use crate::application::delete_usecase::states::{Initial, Validated};
use crate::application::{SupplierBusinessErrors, SupplierPersistence};
use async_trait::async_trait;
use shared::entities::{BusinessError, Context};

pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}
impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        if !request.id.is_valid() {
            return Err(SupplierBusinessErrors::not_found(request.id.value()));
        }
        if request.updated_by.trim().is_empty() {
            return Err(SupplierBusinessErrors::updater_required());
        }
        Ok(Context::new(ValidatedContextData { request }))
    }
}
#[async_trait]
pub trait DeleteSupplierContext {
    async fn delete(self, persistence: &SupplierPersistence) -> Result<(), BusinessError>;
}
#[async_trait]
impl DeleteSupplierContext for Context<Validated, ValidatedContextData> {
    async fn delete(self, persistence: &SupplierPersistence) -> Result<(), BusinessError> {
        persistence
            .delete(
                self.data.request.id.value(),
                self.data.request.updated_by.trim().to_owned(),
            )
            .await
    }
}
