use crate::application::ClientPersistence;
use crate::application::by_id_usecase::data::{GetClientByIdRequest, InitialContextData};
use crate::application::by_id_usecase::states::Initial;
use crate::application::by_id_usecase::steps::{FindClientByIdContext, ValidateInitialContext};
use crate::domain::Client;
use async_trait::async_trait;
use shared::entities::{AsyncGetByIdUseCase, BusinessError, Context};
#[derive(Clone)]
pub struct GetClientByIdService {
    persistence: ClientPersistence,
}
impl GetClientByIdService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }
    pub async fn execute(&self, request: GetClientByIdRequest) -> Result<Client, BusinessError> {
        <Self as AsyncGetByIdUseCase<GetClientByIdRequest, Client>>::execute(self, request).await
    }
}
#[async_trait]
impl AsyncGetByIdUseCase<GetClientByIdRequest, Client> for GetClientByIdService {
    async fn execute(&self, request: GetClientByIdRequest) -> Result<Client, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .find(&self.persistence)
            .await
    }
}
