use crate::application::OrganizationPersistence;
use crate::application::link_client_organization_usecase::data::{
    InitialContextData, LinkClientOrganizationRequest,
};
use crate::application::link_client_organization_usecase::states::Initial;
use crate::application::link_client_organization_usecase::steps::{
    LinkClientOrganizationContext, ValidateInitialContext,
};
use crate::application::service_constructors::FromPersistence;
use crate::domain::OrganizationLink;
use async_trait::async_trait;
use shared::entities::{AsyncAddEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct LinkClientOrganizationService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for LinkClientOrganizationService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl LinkClientOrganizationService {
    pub async fn execute(
        &self,
        request: LinkClientOrganizationRequest,
    ) -> Result<OrganizationLink, BusinessError> {
        <Self as AsyncAddEntityUseCase<LinkClientOrganizationRequest, OrganizationLink>>::execute(
            self, request,
        )
        .await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<LinkClientOrganizationRequest, OrganizationLink>
    for LinkClientOrganizationService
{
    async fn execute(
        &self,
        request: LinkClientOrganizationRequest,
    ) -> Result<OrganizationLink, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .link(&self.persistence)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ClientId, Organization, OrganizationEntityType, OrganizationId};
    use crate::errors::OrganizationBusinessErrors;
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};

    fn make_service_with_org(
        org_id: OrganizationId,
    ) -> (LinkClientOrganizationService, InMemoryOrganizationRepository) {
        let repo = InMemoryOrganizationRepository::new();
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Teste", doc)
            .with_audit(AuditFields::created("admin"));

        repo.insert_for_test(org);

        let service = LinkClientOrganizationService::new_for_tests(repo.clone());
        (service, repo)
    }

    #[tokio::test]
    async fn test_link_client_organization_success() {
        let org_id = OrganizationId::new(TSID::from(100u64));
        let client_id = ClientId::new(TSID::from(200u64));
        let (service, repo) = make_service_with_org(org_id);

        let request = LinkClientOrganizationRequest::new(org_id, client_id, "client-consumer");
        let result = service.execute(request).await;

        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.organization_id(), org_id);
        assert_eq!(link.entity_id(), *client_id);
        assert_eq!(link.entity_type(), OrganizationEntityType::Client);
        assert_eq!(link.created_by(), "client-consumer");
        assert_eq!(repo.links().len(), 1);
    }

    #[tokio::test]
    async fn test_link_client_organization_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let service = LinkClientOrganizationService::new_for_tests(repo);

        let org_id = OrganizationId::new(TSID::from(999u64));
        let client_id = ClientId::new(TSID::from(200u64));
        let request = LinkClientOrganizationRequest::new(org_id, client_id, "client-consumer");

        let result = service.execute(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_link_client_organization_already_exists() {
        let org_id = OrganizationId::new(TSID::from(100u64));
        let client_id = ClientId::new(TSID::from(200u64));
        let (service, _repo) = make_service_with_org(org_id);

        let request = LinkClientOrganizationRequest::new(org_id, client_id, "client-consumer");
        let result1 = service.execute(request.clone()).await;
        assert!(result1.is_ok());

        let result2 = service.execute(request).await;
        assert!(result2.is_err());
        assert_eq!(
            result2.unwrap_err(),
            OrganizationBusinessErrors::already_exists()
        );
    }

    #[tokio::test]
    async fn test_link_client_organization_creator_required() {
        let org_id = OrganizationId::new(TSID::from(100u64));
        let client_id = ClientId::new(TSID::from(200u64));
        let (service, _repo) = make_service_with_org(org_id);

        let request = LinkClientOrganizationRequest::new(org_id, client_id, "");
        let result = service.execute(request).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            OrganizationBusinessErrors::creator_required()
        );
    }
}
