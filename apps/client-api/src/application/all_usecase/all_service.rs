use crate::application::ClientPersistence;
use crate::application::all_usecase::data::{FindClientsRequest, InitialContextData};
use crate::application::all_usecase::states::Initial;
use crate::application::all_usecase::steps::{FindClientsContext, ValidateInitialContext};
use crate::domain::Client;
use async_trait::async_trait;
use shared::entities::{
    BusinessError,
    use_case::{context::Context, use_case_traits::AsyncGetAllUseCase},
};
use shared::types::Page;
#[derive(Clone)]
pub struct FindClientsService {
    persistence: ClientPersistence,
}
impl FindClientsService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }
    pub async fn execute(
        &self,
        request: FindClientsRequest,
    ) -> Result<Page<Client>, BusinessError> {
        <Self as AsyncGetAllUseCase<FindClientsRequest, Page<Client>>>::execute(self, request).await
    }
}
#[async_trait]
impl AsyncGetAllUseCase<FindClientsRequest, Page<Client>> for FindClientsService {
    async fn execute(&self, request: FindClientsRequest) -> Result<Page<Client>, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .find(&self.persistence)
            .await
    }
}
