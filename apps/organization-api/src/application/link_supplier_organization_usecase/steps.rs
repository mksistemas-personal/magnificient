use crate::application::OrganizationBusinessErrors;
use crate::application::OrganizationPersistence;
use crate::application::link_supplier_organization_usecase::data::{
    InitialContextData, ValidatedContextData,
};
use crate::application::link_supplier_organization_usecase::states::{Initial, Validated};
use crate::domain::{OrganizationEntityType, OrganizationLink};
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
        if !request.supplier_id.is_valid() {
            return Err(OrganizationBusinessErrors::id_invalid());
        }
        if request.created_by.trim().is_empty() {
            return Err(OrganizationBusinessErrors::creator_required());
        }

        Ok(Context::new(ValidatedContextData { request }))
    }
}

#[async_trait]
pub trait LinkSupplierOrganizationContext {
    async fn link(
        self,
        persistence: &OrganizationPersistence,
    ) -> Result<OrganizationLink, BusinessError>;
}

#[async_trait]
impl LinkSupplierOrganizationContext for Context<Validated, ValidatedContextData> {
    async fn link(
        self,
        persistence: &OrganizationPersistence,
    ) -> Result<OrganizationLink, BusinessError> {
        let link = OrganizationLink::new(
            self.data.request.organization_id,
            *self.data.request.supplier_id,
            OrganizationEntityType::Supplier,
            self.data.request.created_by,
        );
        persistence.link_entity(link).await
    }
}
