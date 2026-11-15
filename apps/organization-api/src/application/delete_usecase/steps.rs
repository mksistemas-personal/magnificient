use crate::application::OrganizationBusinessErrors;
use crate::application::OrganizationPersistence;
use crate::application::delete_usecase::data::{
    InitialContextData, LinksCheckedContextData, ValidatedContextData,
};
use crate::application::delete_usecase::states::{Initial, LinksChecked, Validated};
use crate::domain::LinkedEntity;
use async_trait::async_trait;
use shared::entities::{BusinessError, Context};

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
pub trait CheckOrganizationLinksContext {
    async fn check_links(
        self,
        persistence: &OrganizationPersistence,
    ) -> Result<Context<LinksChecked, LinksCheckedContextData>, BusinessError>;
}

#[async_trait]
impl CheckOrganizationLinksContext for Context<Validated, ValidatedContextData> {
    async fn check_links(
        self,
        persistence: &OrganizationPersistence,
    ) -> Result<Context<LinksChecked, LinksCheckedContextData>, BusinessError> {
        let links = persistence
            .find_links_by_organization_id(*self.data.request.organization_id)
            .await?;

        if !links.is_empty() {
            let linked_entities: Vec<LinkedEntity> =
                links.into_iter().map(LinkedEntity::from).collect();
            return Err(OrganizationBusinessErrors::has_linked_entities(
                &linked_entities,
            ));
        }

        Ok(Context::new(LinksCheckedContextData {
            request: self.data.request,
        }))
    }
}

#[async_trait]
pub trait DeleteOrganizationContext {
    async fn delete(self, persistence: &OrganizationPersistence) -> Result<(), BusinessError>;
}

#[async_trait]
impl DeleteOrganizationContext for Context<LinksChecked, LinksCheckedContextData> {
    async fn delete(self, persistence: &OrganizationPersistence) -> Result<(), BusinessError> {
        persistence
            .delete(
                *self.data.request.organization_id,
                &self.data.request.updated_by,
            )
            .await
    }
}

#[async_trait]
impl DeleteOrganizationContext for Context<Validated, ValidatedContextData> {
    async fn delete(self, persistence: &OrganizationPersistence) -> Result<(), BusinessError> {
        persistence
            .delete(
                *self.data.request.organization_id,
                &self.data.request.updated_by,
            )
            .await
    }
}
