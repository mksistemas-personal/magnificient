use crate::application::OrganizationPersistence;
use crate::application::link_supplier_organization_usecase::data::{
    InitialContextData, LinkSupplierOrganizationRequest,
};
use crate::application::link_supplier_organization_usecase::states::Initial;
use crate::application::link_supplier_organization_usecase::steps::{
    LinkSupplierOrganizationContext, ValidateInitialContext,
};
use crate::application::service_constructors::FromPersistence;
use crate::domain::OrganizationLink;
use async_trait::async_trait;
use shared::entities::{AsyncAddEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct LinkSupplierOrganizationService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for LinkSupplierOrganizationService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl LinkSupplierOrganizationService {
    pub async fn execute(
        &self,
        request: LinkSupplierOrganizationRequest,
    ) -> Result<OrganizationLink, BusinessError> {
        <Self as AsyncAddEntityUseCase<LinkSupplierOrganizationRequest, OrganizationLink>>::execute(
            self, request,
        )
        .await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<LinkSupplierOrganizationRequest, OrganizationLink>
    for LinkSupplierOrganizationService
{
    async fn execute(
        &self,
        request: LinkSupplierOrganizationRequest,
    ) -> Result<OrganizationLink, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .link(&self.persistence)
            .await
    }
}
