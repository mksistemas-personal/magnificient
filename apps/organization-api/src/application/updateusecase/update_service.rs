use crate::application::updateusecase::data::*;
use crate::application::updateusecase::states::*;
use crate::application::updateusecase::steps::{
    ChangeOrganizationContext, LoadOrganizationContext, PersistOrganizationContext,
    ValidateInitialContext,
};
use crate::domain::{Organization, OrganizationRepository};
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::BusinessError;
use shared::entities::use_case::context::Context;
use shared::entities::use_case::use_case_traits::AsyncUpdateEntityUseCase;
use std::sync::Arc;

#[derive(Clone)]
pub struct UpdateOrganizationService {
    repository: Arc<dyn OrganizationRepository>,
    uow: Option<UnitOfWork>,
}

impl UpdateOrganizationService {
    pub fn new(repository: impl OrganizationRepository + 'static) -> Self {
        Self {
            repository: Arc::new(repository),
            uow: None,
        }
    }

    pub fn new_with_uow(
        repository: impl OrganizationRepository + 'static,
        uow: UnitOfWork,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            uow: Some(uow),
        }
    }

    pub async fn execute(
        &self,
        request: UpdateOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        <Self as AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>>::execute(
            self, request,
        )
        .await
    }
}

#[async_trait]
impl AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>
    for UpdateOrganizationService
{
    async fn execute(
        &self,
        request: UpdateOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .load(self.repository.as_ref())
            .await?
            .change()?
            .save(self.repository.as_ref(), self.uow.as_ref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::OrganizationId;
    use crate::errors::OrganizationBusinessErrors;
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};

    #[tokio::test]
    async fn test_update_organization_service_full_flow() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Nome Original", doc)
            .with_audit(AuditFields::created("admin"));

        repo.save(&org).await.unwrap();

        let service = UpdateOrganizationService::new(repo);

        let update_req = UpdateOrganizationRequest {
            organization_id: org_id,
            name: "Novo Nome".to_string(),
            updated_by: "editor".to_string(),
        };

        let result = service.execute(update_req).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name(), "Novo Nome");
        let audit = updated.audit.as_ref().unwrap();
        assert_eq!(audit.created_by(), "admin");
        assert_eq!(audit.updated_by(), "editor");
    }

    #[tokio::test]
    async fn test_update_organization_service_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let service = UpdateOrganizationService::new(repo);

        let update_req = UpdateOrganizationRequest {
            organization_id: OrganizationId::new(TSID::from(999u64)),
            name: "Empresa Inexistente".to_string(),
            updated_by: "editor".to_string(),
        };

        let result = service.execute(update_req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_update_organization_validation_errors() {
        let repo = InMemoryOrganizationRepository::new();
        let service = UpdateOrganizationService::new(repo);
        let org_id = OrganizationId::new(TSID::from(100u64));

        let invalid_name_req = UpdateOrganizationRequest {
            organization_id: org_id,
            name: "".to_string(),
            updated_by: "editor".to_string(),
        };
        assert_eq!(
            service.execute(invalid_name_req).await.unwrap_err(),
            OrganizationBusinessErrors::name_empty()
        );

        let invalid_updater_req = UpdateOrganizationRequest {
            organization_id: org_id,
            name: "Nome".to_string(),
            updated_by: "".to_string(),
        };
        assert_eq!(
            service.execute(invalid_updater_req).await.unwrap_err(),
            OrganizationBusinessErrors::updater_required()
        );
    }
}
