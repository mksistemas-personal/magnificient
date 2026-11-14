use crate::application::ClientPersistence;
use crate::application::delete_usecase::data::{DeleteClientRequest, InitialContextData};
use crate::application::delete_usecase::states::Initial;
use crate::application::delete_usecase::steps::{DeleteClientContext, ValidateInitialContext};
use async_trait::async_trait;
use shared::entities::{AsyncDeleteEntityUseCase, BusinessError, Context};
#[derive(Clone)]
pub struct DeleteClientService {
    persistence: ClientPersistence,
}
impl DeleteClientService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }
    pub async fn execute(&self, request: DeleteClientRequest) -> Result<(), BusinessError> {
        <Self as AsyncDeleteEntityUseCase<DeleteClientRequest, ()>>::execute(self, request).await
    }
}
#[async_trait]
impl AsyncDeleteEntityUseCase<DeleteClientRequest, ()> for DeleteClientService {
    async fn execute(&self, request: DeleteClientRequest) -> Result<(), BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .delete(&self.persistence)
            .await
    }
}
