use crate::application::OrganizationPersistence;
use crate::application::service_constructors::FromPersistence;
use crate::application::unlink_supplier_organization_usecase::data::{
    InitialContextData, UnlinkSupplierOrganizationRequest,
};
use crate::application::unlink_supplier_organization_usecase::states::Initial;
use crate::application::unlink_supplier_organization_usecase::steps::{
    UnlinkSupplierOrganizationContext, ValidateInitialContext,
};
use async_trait::async_trait;
use shared::entities::{AsyncDeleteEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct UnlinkSupplierOrganizationService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for UnlinkSupplierOrganizationService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl UnlinkSupplierOrganizationService {
    pub async fn execute(
        &self,
        request: UnlinkSupplierOrganizationRequest,
    ) -> Result<(), BusinessError> {
        <Self as AsyncDeleteEntityUseCase<UnlinkSupplierOrganizationRequest, ()>>::execute(
            self, request,
        )
        .await
    }
}

#[async_trait]
impl AsyncDeleteEntityUseCase<UnlinkSupplierOrganizationRequest, ()>
    for UnlinkSupplierOrganizationService
{
    async fn execute(
        &self,
        request: UnlinkSupplierOrganizationRequest,
    ) -> Result<(), BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .unlink(&self.persistence)
            .await
    }
}
