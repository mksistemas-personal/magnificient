use crate::application::ClientPersistence;
use crate::application::add_usecase::data::{CreateClientRequest, InitialContextData};
use crate::application::add_usecase::states::Initial;
use crate::application::add_usecase::steps::{
    BuildClientContext, PersistClientContext, ValidateAddressContext, ValidateInitialContext,
};
use crate::domain::Client;
use async_trait::async_trait;
use shared::entities::{AsyncAddEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct CreateClientService {
    persistence: ClientPersistence,
}

impl CreateClientService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(&self, request: CreateClientRequest) -> Result<Client, BusinessError> {
        <Self as AsyncAddEntityUseCase<CreateClientRequest, Client>>::execute(self, request).await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<CreateClientRequest, Client> for CreateClientService {
    async fn execute(&self, request: CreateClientRequest) -> Result<Client, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .validate_address()?
            .build()?
            .save(&self.persistence)
            .await
    }
}
