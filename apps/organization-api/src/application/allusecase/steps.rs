use crate::application::allusecase::data::{InitialContextData, ValidatedContextData};
use crate::application::allusecase::states::{Initial, Validated};
use crate::domain::{Organization, OrganizationRepository};
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::BusinessError;
use shared::entities::use_case::context::Context;
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
        repository: &dyn OrganizationRepository,
        uow: Option<&UnitOfWork>,
    ) -> Result<Page<Organization>, BusinessError>;
}

#[async_trait]
impl FindOrganizationsContext for Context<Validated, ValidatedContextData> {
    async fn find(
        self,
        repository: &dyn OrganizationRepository,
        uow: Option<&UnitOfWork>,
    ) -> Result<Page<Organization>, BusinessError> {
        let name_filter = self.data.request.name.as_deref();
        let page_req = self.data.request.page_request;
        UnitOfWork::execute_maybe_tx(uow, || async {
            let content = repository.find_all(name_filter, &page_req).await?;
            let total = repository.count(name_filter).await?;
            Ok(Page::new(content, page_req.page, page_req.page_size, total))
        })
        .await
    }
}
