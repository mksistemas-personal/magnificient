use crate::application::{LinkClientOrganizationRequest, OrganizationPersistence};
use shared::entities::BusinessError;

#[derive(Clone)]
pub struct LinkClientOrganizationService {
    persistence: OrganizationPersistence,
}

impl LinkClientOrganizationService {
    pub fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(
        &self,
        request: LinkClientOrganizationRequest,
    ) -> Result<(), BusinessError> {
        Ok(())
    }
}
