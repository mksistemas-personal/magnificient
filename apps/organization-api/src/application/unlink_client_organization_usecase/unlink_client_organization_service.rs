use crate::application::OrganizationPersistence;
use crate::application::service_constructors::FromPersistence;
use crate::application::unlink_client_organization_usecase::data::{
    InitialContextData, UnlinkClientOrganizationRequest,
};
use crate::application::unlink_client_organization_usecase::states::Initial;
use crate::application::unlink_client_organization_usecase::steps::{
    UnlinkClientOrganizationContext, ValidateInitialContext,
};
use async_trait::async_trait;
use shared::entities::{AsyncDeleteEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct UnlinkClientOrganizationService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for UnlinkClientOrganizationService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl UnlinkClientOrganizationService {
    pub async fn execute(
        &self,
        request: UnlinkClientOrganizationRequest,
    ) -> Result<(), BusinessError> {
        <Self as AsyncDeleteEntityUseCase<UnlinkClientOrganizationRequest, ()>>::execute(
            self, request,
        )
        .await
    }
}

#[async_trait]
impl AsyncDeleteEntityUseCase<UnlinkClientOrganizationRequest, ()>
    for UnlinkClientOrganizationService
{
    async fn execute(
        &self,
        request: UnlinkClientOrganizationRequest,
    ) -> Result<(), BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .unlink(&self.persistence)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ClientId, Organization, OrganizationEntityType, OrganizationId, OrganizationLink,
    };
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};

    #[tokio::test]
    async fn test_unlink_client_organization_success() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Teste", doc)
            .with_audit(AuditFields::created("admin"));

        repo.insert_for_test(org);

        let client_id = ClientId::new(TSID::from(200u64));
        let link =
            OrganizationLink::new(org_id, *client_id, OrganizationEntityType::Client, "test");
        repo.insert_link_for_test(link);

        assert_eq!(repo.links().len(), 1);

        let service = UnlinkClientOrganizationService::new_for_tests(repo.clone());
        let request = UnlinkClientOrganizationRequest::new(client_id);
        let result = service.execute(request).await;

        assert!(result.is_ok());
        assert_eq!(repo.links().len(), 0);
    }

    #[tokio::test]
    async fn test_unlink_client_organization_when_not_linked_is_ok() {
        let repo = InMemoryOrganizationRepository::new();
        let service = UnlinkClientOrganizationService::new_for_tests(repo.clone());
        let client_id = ClientId::new(TSID::from(999u64));
        let request = UnlinkClientOrganizationRequest::new(client_id);
        let result = service.execute(request).await;

        assert!(result.is_ok());
    }
}
