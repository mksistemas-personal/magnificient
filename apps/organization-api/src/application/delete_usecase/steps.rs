use crate::application::OrganizationBusinessErrors;
use crate::application::delete_usecase::data::{InitialContextData, ValidatedContextData};
use crate::application::delete_usecase::states::{Initial, Validated};
use crate::domain::OrganizationRepository;
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::BusinessError;
use shared::entities::use_case::context::Context;

pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;

        if !request.organization_id.is_valid() {
            return Err(OrganizationBusinessErrors::id_invalid());
        }
        if request.updated_by.trim().is_empty() {
            return Err(OrganizationBusinessErrors::updater_required());
        }

        Ok(Context::new(ValidatedContextData { request }))
    }
}

#[async_trait]
pub trait DeleteOrganizationContext {
    async fn delete(
        self,
        repository: &dyn OrganizationRepository,
        uow: Option<&UnitOfWork>,
    ) -> Result<(), BusinessError>;
}

#[async_trait]
impl DeleteOrganizationContext for Context<Validated, ValidatedContextData> {
    async fn delete(
        self,
        repository: &dyn OrganizationRepository,
        uow: Option<&UnitOfWork>,
    ) -> Result<(), BusinessError> {
        let org_id = self.data.request.organization_id;
        let updated_by = self.data.request.updated_by;
        UnitOfWork::execute_maybe_tx(uow, || async {
            repository.delete(*org_id, &updated_by).await
        })
        .await
    }
}
