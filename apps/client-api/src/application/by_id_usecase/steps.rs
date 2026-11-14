use crate::application::by_id_usecase::data::{InitialContextData, ValidatedContextData};
use crate::application::by_id_usecase::states::{Initial, Validated};
use crate::application::{ClientBusinessErrors, ClientPersistence};
use crate::domain::Client;
use async_trait::async_trait;
use shared::entities::{BusinessError, Context};
pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}
impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        if !self.data.request.id.is_valid() {
            return Err(ClientBusinessErrors::not_found(
                self.data.request.id.value(),
            ));
        }
        Ok(Context::new(ValidatedContextData {
            request: self.data.request,
        }))
    }
}
#[async_trait]
pub trait FindClientByIdContext {
    async fn find(self, persistence: &ClientPersistence) -> Result<Client, BusinessError>;
}
#[async_trait]
impl FindClientByIdContext for Context<Validated, ValidatedContextData> {
    async fn find(self, persistence: &ClientPersistence) -> Result<Client, BusinessError> {
        persistence.find_by_id(self.data.request.id.value()).await
    }
}
