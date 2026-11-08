use crate::application::OrganizationPersistence;
use crate::application::by_id_usecase::data::{FindOrganizationByIdRequest, InitialContextData};
use crate::application::by_id_usecase::states::Initial;
use crate::application::by_id_usecase::steps::{
    FindOrganizationByIdContext, ValidateInitialContext,
};
use crate::application::service_constructors::FromPersistence;
use crate::domain::Organization;
use async_trait::async_trait;
use shared::entities::BusinessError;
use shared::entities::use_case::context::Context;
use shared::entities::use_case::use_case_traits::AsyncGetByIdUseCase;

#[derive(Clone)]
pub struct FindOrganizationByIdService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for FindOrganizationByIdService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl FindOrganizationByIdService {
    pub async fn execute(
        &self,
        request: FindOrganizationByIdRequest,
    ) -> Result<Organization, BusinessError> {
        <Self as AsyncGetByIdUseCase<FindOrganizationByIdRequest, Organization>>::execute(
            self, request,
        )
        .await
    }
}

pub type GetOrganizationByIdService = FindOrganizationByIdService;

#[async_trait]
impl AsyncGetByIdUseCase<FindOrganizationByIdRequest, Organization>
    for FindOrganizationByIdService
{
    async fn execute(
        &self,
        request: FindOrganizationByIdRequest,
    ) -> Result<Organization, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .find(&self.persistence)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::OrganizationBusinessErrors;
    use crate::application::service_constructors::FromPersistence;
    use crate::domain::{Organization, OrganizationId, OrganizationRepository};
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};

    #[tokio::test]
    async fn test_find_organization_by_id_service_full_flow() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Leitura", doc)
            .with_audit(AuditFields::created("admin"));

        let conn: &mut sqlx::PgConnection = unsafe { &mut *std::ptr::NonNull::dangling().as_ptr() };
        OrganizationRepository::save(&repo, conn, &org)
            .await
            .unwrap();

        let service = FindOrganizationByIdService::new_for_tests(repo);
        let request = FindOrganizationByIdRequest::new(org_id);

        let result = service.execute(request).await;
        assert!(result.is_ok());

        let found_org = result.unwrap();
        assert_eq!(found_org.id(), org_id);
        assert_eq!(found_org.name(), "Empresa Leitura");
    }

    #[tokio::test]
    async fn test_find_organization_by_id_service_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let service = FindOrganizationByIdService::new_for_tests(repo);

        let request = FindOrganizationByIdRequest::new(OrganizationId::new(TSID::from(999u64)));

        let result = service.execute(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_find_organization_by_id_validation_errors() {
        let repo = InMemoryOrganizationRepository::new();
        let service = FindOrganizationByIdService::new_for_tests(repo);

        let invalid_req = FindOrganizationByIdRequest::new(OrganizationId::new(TSID::from(0u64)));
        let result = service.execute(invalid_req).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            OrganizationBusinessErrors::id_invalid()
        );
    }
}
