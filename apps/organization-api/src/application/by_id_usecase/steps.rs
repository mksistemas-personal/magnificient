use crate::application::OrganizationBusinessErrors;
use crate::application::by_id_usecase::data::{InitialContextData, ValidatedContextData};
use crate::application::by_id_usecase::states::{Initial, Validated};
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

        Ok(Context::new(ValidatedContextData { request }))
    }
}

#[async_trait]
pub trait FindOrganizationByIdContext {
    async fn find(
        self,
        repository: &dyn OrganizationRepository,
        uow: Option<&UnitOfWork>,
    ) -> Result<Organization, BusinessError>;
}

#[async_trait]
impl FindOrganizationByIdContext for Context<Validated, ValidatedContextData> {
    async fn find(
        self,
        repository: &dyn OrganizationRepository,
        uow: Option<&UnitOfWork>,
    ) -> Result<Organization, BusinessError> {
        let org_id = self.data.request.organization_id;
        match uow {
            Some(uow) => {
                uow.run_tx(|mut tx| async move {
                    let organization = repository.find_by_id_in_tx(&mut tx, *org_id).await?;
                    Ok((organization, tx))
                })
                .await
            }
            None => repository.find_by_id(*org_id).await,
        }
    }
}
