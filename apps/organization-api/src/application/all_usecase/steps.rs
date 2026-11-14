use crate::application::OrganizationPersistence;
use crate::application::all_usecase::data::{InitialContextData, ValidatedContextData};
use crate::application::all_usecase::states::{Initial, Validated};
use crate::domain::Organization;
use async_trait::async_trait;
use shared::entities::{BusinessError, Context};
use shared::types::Page;

pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        Ok(Context::new(ValidatedContextData { request }))
    }
}

#[async_trait]
pub trait FindOrganizationsContext {
    async fn find(
        self,
        persistence: &OrganizationPersistence,
    ) -> Result<Page<Organization>, BusinessError>;
}

#[async_trait]
impl FindOrganizationsContext for Context<Validated, ValidatedContextData> {
    async fn find(
        self,
        persistence: &OrganizationPersistence,
    ) -> Result<Page<Organization>, BusinessError> {
        persistence
            .find_page(
                self.data.request.name.as_deref(),
                self.data.request.page_request,
            )
            .await
    }
}
