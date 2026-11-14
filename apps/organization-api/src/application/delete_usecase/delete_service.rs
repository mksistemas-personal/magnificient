use crate::application::OrganizationPersistence;
use crate::application::delete_usecase::data::*;
use crate::application::delete_usecase::states::*;
use crate::application::delete_usecase::steps::{
    DeleteOrganizationContext, ValidateInitialContext,
};
use crate::application::service_constructors::FromPersistence;
use async_trait::async_trait;
use shared::entities::{AsyncDeleteEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct DeleteOrganizationService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for DeleteOrganizationService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl DeleteOrganizationService {
    pub async fn execute(&self, request: DeleteOrganizationRequest) -> Result<(), BusinessError> {
        <Self as AsyncDeleteEntityUseCase<DeleteOrganizationRequest, ()>>::execute(self, request)
            .await
    }
}

#[async_trait]
impl AsyncDeleteEntityUseCase<DeleteOrganizationRequest, ()> for DeleteOrganizationService {
    async fn execute(&self, request: DeleteOrganizationRequest) -> Result<(), BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .delete(&self.persistence)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service_constructors::FromPersistence;
    use crate::domain::{Organization, OrganizationId, OrganizationRepository};
    use crate::errors::OrganizationBusinessErrors;
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};

    async fn seed(repo: &InMemoryOrganizationRepository, id: u64) {
        let org_id = OrganizationId::new(TSID::from(id));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Para Deletar", doc)
            .with_audit(AuditFields::created("admin"));
        let conn: &mut sqlx::PgConnection = unsafe { &mut *std::ptr::NonNull::dangling().as_ptr() };
        OrganizationRepository::save(repo, conn, &org)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_organization_service_full_flow() {
        let repo = InMemoryOrganizationRepository::new();
        seed(&repo, 100).await;

        let service = DeleteOrganizationService::new_for_tests(repo.clone());

        let delete_req = DeleteOrganizationRequest {
            organization_id: OrganizationId::new(TSID::from(100u64)),
            updated_by: "deleter_admin".to_string(),
        };

        let result = service.execute(delete_req).await;
        assert!(result.is_ok());

        let conn: &mut sqlx::PgConnection = unsafe { &mut *std::ptr::NonNull::dangling().as_ptr() };
        let find_res = OrganizationRepository::find_by_id(&repo, conn, TSID::from(100u64)).await;
        assert!(find_res.is_err());
        assert_eq!(
            find_res.unwrap_err(),
            OrganizationBusinessErrors::not_found()
        );
    }

    #[tokio::test]
    async fn test_delete_organization_service_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let service = DeleteOrganizationService::new_for_tests(repo);

        let delete_req = DeleteOrganizationRequest {
            organization_id: OrganizationId::new(TSID::from(999u64)),
            updated_by: "deleter_admin".to_string(),
        };

        let result = service.execute(delete_req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_delete_organization_validation_errors() {
        let repo = InMemoryOrganizationRepository::new();
        let service = DeleteOrganizationService::new_for_tests(repo);
        let org_id = OrganizationId::new(TSID::from(100u64));

        let invalid_updater_req = DeleteOrganizationRequest {
            organization_id: org_id,
            updated_by: "   ".to_string(),
        };
        assert_eq!(
            service.execute(invalid_updater_req).await.unwrap_err(),
            OrganizationBusinessErrors::updater_required()
        );
    }
}
