use crate::application::all_usecase::data::{InitialContextData, ValidatedContextData};
use crate::application::all_usecase::states::{Initial, Validated};
use crate::application::{ClientBusinessErrors, ClientPersistence};
use crate::domain::Client;
use async_trait::async_trait;
use shared::entities::{BusinessError, use_case::context::Context};
use shared::types::Page;
pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}
impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        if request.page_request.page == 0 || request.page_request.page_size == 0 {
            return Err(ClientBusinessErrors::pagination_invalid());
        }
        if request.organization_id.is_some_and(|id| !id.is_valid()) {
            return Err(ClientBusinessErrors::organization_required());
        }
        Ok(Context::new(ValidatedContextData { request }))
    }
}
#[async_trait]
pub trait FindClientsContext {
    async fn find(self, persistence: &ClientPersistence) -> Result<Page<Client>, BusinessError>;
}
#[async_trait]
impl FindClientsContext for Context<Validated, ValidatedContextData> {
    async fn find(self, persistence: &ClientPersistence) -> Result<Page<Client>, BusinessError> {
        persistence
            .find_page(
                self.data.request.name,
                self.data.request.organization_id.map(|id| id.value()),
                self.data.request.page_request,
            )
            .await
    }
}
