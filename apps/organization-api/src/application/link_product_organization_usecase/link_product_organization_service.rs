use crate::application::OrganizationPersistence;
use crate::application::link_product_organization_usecase::data::{
    InitialContextData, LinkProductOrganizationRequest,
};
use crate::application::link_product_organization_usecase::states::Initial;
use crate::application::link_product_organization_usecase::steps::{
    LinkProductOrganizationContext, ValidateInitialContext,
};
use crate::application::service_constructors::FromPersistence;
use crate::domain::OrganizationLink;
use async_trait::async_trait;
use shared::entities::{AsyncAddEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct LinkProductOrganizationService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for LinkProductOrganizationService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl LinkProductOrganizationService {
    pub async fn execute(
        &self,
        request: LinkProductOrganizationRequest,
    ) -> Result<OrganizationLink, BusinessError> {
        <Self as AsyncAddEntityUseCase<LinkProductOrganizationRequest, OrganizationLink>>::execute(
            self, request,
        )
        .await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<LinkProductOrganizationRequest, OrganizationLink>
    for LinkProductOrganizationService
{
    async fn execute(
        &self,
        request: LinkProductOrganizationRequest,
    ) -> Result<OrganizationLink, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .link(&self.persistence)
            .await
    }
}
