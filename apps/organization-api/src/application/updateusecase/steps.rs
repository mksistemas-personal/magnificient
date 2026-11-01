use crate::application::OrganizationBusinessErrors;
use crate::application::updateusecase::data::{
    ChangedContextData, InitialContextData, LoadedContextData, ValidatedContextData,
};
use crate::application::updateusecase::states::{Changed, Initial, Loaded, Validated};
use crate::domain::{Organization, OrganizationRepository};
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
        if request.name.trim().is_empty() {
            return Err(OrganizationBusinessErrors::name_empty());
        }
        if request.updated_by.trim().is_empty() {
            return Err(OrganizationBusinessErrors::updater_required());
        }

        Ok(Context::new(ValidatedContextData { request }))
    }
}

#[async_trait]
pub trait LoadOrganizationContext {
    async fn load(
        self,
        repository: &dyn OrganizationRepository,
    ) -> Result<Context<Loaded, LoadedContextData>, BusinessError>;
}

#[async_trait]
impl LoadOrganizationContext for Context<Validated, ValidatedContextData> {
    async fn load(
        self,
        repository: &dyn OrganizationRepository,
    ) -> Result<Context<Loaded, LoadedContextData>, BusinessError> {
        let org_id = self.data.request.organization_id;
        let organization = repository.find_by_id(*org_id).await?;

        Ok(Context::new(LoadedContextData {
            request: self.data.request,
            organization,
        }))
    }
}

pub trait ChangeOrganizationContext {
    fn change(self) -> Result<Context<Changed, ChangedContextData>, BusinessError>;
}

impl ChangeOrganizationContext for Context<Loaded, LoadedContextData> {
    fn change(mut self) -> Result<Context<Changed, ChangedContextData>, BusinessError> {
        let name = self.data.request.name;
        let updated_by = self.data.request.updated_by;

        self.data.organization.update(name, updated_by);

        Ok(Context::new(ChangedContextData {
            organization: self.data.organization,
        }))
    }
}

#[async_trait]
pub trait PersistOrganizationContext {
    async fn save(
        self,
        repository: &dyn OrganizationRepository,
        uow: Option<&UnitOfWork>,
    ) -> Result<Organization, BusinessError>;
}

#[async_trait]
impl PersistOrganizationContext for Context<Changed, ChangedContextData> {
    async fn save(
        self,
        repository: &dyn OrganizationRepository,
        uow: Option<&UnitOfWork>,
    ) -> Result<Organization, BusinessError> {
        let organization = self.data.organization;
        UnitOfWork::execute_maybe_tx(uow, || async { repository.save(&organization).await }).await
    }
}
