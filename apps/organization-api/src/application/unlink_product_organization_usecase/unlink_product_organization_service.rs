use crate::application::OrganizationPersistence;
use crate::application::service_constructors::FromPersistence;
use crate::application::unlink_product_organization_usecase::data::{
    InitialContextData, UnlinkProductOrganizationRequest,
};
use crate::application::unlink_product_organization_usecase::states::Initial;
use crate::application::unlink_product_organization_usecase::steps::{
    UnlinkProductOrganizationContext, ValidateInitialContext,
};
use async_trait::async_trait;
use shared::entities::{AsyncDeleteEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct UnlinkProductOrganizationService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for UnlinkProductOrganizationService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl UnlinkProductOrganizationService {
    pub async fn execute(
        &self,
        request: UnlinkProductOrganizationRequest,
    ) -> Result<(), BusinessError> {
        <Self as AsyncDeleteEntityUseCase<UnlinkProductOrganizationRequest, ()>>::execute(
            self, request,
        )
        .await
    }
}

#[async_trait]
impl AsyncDeleteEntityUseCase<UnlinkProductOrganizationRequest, ()>
    for UnlinkProductOrganizationService
{
    async fn execute(
        &self,
        request: UnlinkProductOrganizationRequest,
    ) -> Result<(), BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .unlink(&self.persistence)
            .await
    }
}
