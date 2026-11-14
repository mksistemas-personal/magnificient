use crate::application::ClientPersistence;
use crate::application::update_usecase::data::{InitialContextData, UpdateClientRequest};
use crate::application::update_usecase::states::Initial;
use crate::application::update_usecase::steps::{
    ChangeClientContext, LoadClientContext, PersistClientContext, ValidateAddressContext,
    ValidateInitialContext,
};
use crate::domain::Client;
use async_trait::async_trait;
use shared::entities::{AsyncUpdateEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct UpdateClientService {
    persistence: ClientPersistence,
}

impl UpdateClientService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(&self, request: UpdateClientRequest) -> Result<Client, BusinessError> {
        <Self as AsyncUpdateEntityUseCase<UpdateClientRequest, Client>>::execute(self, request)
            .await
    }
}

#[async_trait]
impl AsyncUpdateEntityUseCase<UpdateClientRequest, Client> for UpdateClientService {
    async fn execute(&self, request: UpdateClientRequest) -> Result<Client, BusinessError> {
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
