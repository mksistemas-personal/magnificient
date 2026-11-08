use crate::application::delete_usecase::data::{InitialContextData, ValidatedContextData};
use crate::application::delete_usecase::states::{Initial, Validated};
use crate::application::{ClientBusinessErrors, ClientPersistence};
use async_trait::async_trait;
use shared::entities::{BusinessError, use_case::context::Context};

pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}
impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        if !request.id.is_valid() {
            return Err(ClientBusinessErrors::not_found(request.id.value()));
        }
        if request.updated_by.trim().is_empty() {
            return Err(ClientBusinessErrors::updater_required());
        }
        Ok(Context::new(ValidatedContextData { request }))
    }
}
#[async_trait]
pub trait DeleteClientContext {
    async fn delete(self, persistence: &ClientPersistence) -> Result<(), BusinessError>;
}
#[async_trait]
impl DeleteClientContext for Context<Validated, ValidatedContextData> {
    async fn delete(self, persistence: &ClientPersistence) -> Result<(), BusinessError> {
        persistence
            .delete(
                self.data.request.id.value(),
                self.data.request.updated_by.trim().to_owned(),
            )
            .await
    }
}
