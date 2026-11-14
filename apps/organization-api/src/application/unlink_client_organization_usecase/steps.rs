use crate::application::OrganizationBusinessErrors;
use crate::application::OrganizationPersistence;
use crate::application::unlink_client_organization_usecase::data::{
    InitialContextData, ValidatedContextData,
};
use crate::application::unlink_client_organization_usecase::states::{Initial, Validated};
use crate::domain::OrganizationEntityType;
use async_trait::async_trait;
use shared::entities::{BusinessError, Context};

pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;

        if !request.client_id.is_valid() {
            return Err(OrganizationBusinessErrors::id_invalid());
        }

        Ok(Context::new(ValidatedContextData { request }))
    }
}

#[async_trait]
pub trait UnlinkClientOrganizationContext {
    async fn unlink(self, persistence: &OrganizationPersistence) -> Result<(), BusinessError>;
}

#[async_trait]
impl UnlinkClientOrganizationContext for Context<Validated, ValidatedContextData> {
    async fn unlink(self, persistence: &OrganizationPersistence) -> Result<(), BusinessError> {
        persistence
            .unlink_entity(*self.data.request.client_id, OrganizationEntityType::Client)
            .await
    }
}
