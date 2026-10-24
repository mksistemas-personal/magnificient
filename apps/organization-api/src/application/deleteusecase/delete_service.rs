use crate::application::OrganizationBusinessErrors;
use crate::application::deleteusecase::data::*;
use crate::application::deleteusecase::states::*;
use crate::domain::OrganizationRepository;
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::BusinessError;
use shared::entities::use_case::context::Context;
use shared::entities::use_case::use_case_traits::AsyncDeleteEntityUseCase;
use std::sync::Arc;

trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;

        if !request.organization_id.is_valid() {
            return Err(OrganizationBusinessErrors::id_invalid());
        }
        if request.updated_by.trim().is_empty() {
            return Err(OrganizationBusinessErrors::updater_required());
        }

        Ok(Context::new(ValidatedContextData { request }))
    }
}

trait DeleteOrganizationContext {
    async fn delete(self, repository: &dyn OrganizationRepository) -> Result<(), BusinessError>;
}

impl DeleteOrganizationContext for Context<Validated, ValidatedContextData> {
    async fn delete(self, repository: &dyn OrganizationRepository) -> Result<(), BusinessError> {
        let org_id = self.data.request.organization_id;
        let updated_by = self.data.request.updated_by;
        repository.delete(*org_id, &updated_by).await
    }
}

#[derive(Clone)]
pub struct DeleteOrganizationService {
    repository: Arc<dyn OrganizationRepository>,
    uow: Option<UnitOfWork>,
}

impl DeleteOrganizationService {
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

    pub fn new_with_arc(repository: Arc<dyn OrganizationRepository>) -> Self {
        Self {
            repository,
            uow: None,
        }
    }

    pub fn new_with_arc_and_uow(
        repository: Arc<dyn OrganizationRepository>,
        uow: UnitOfWork,
    ) -> Self {
        Self {
            repository,
            uow: Some(uow),
        }
    }

    pub fn with_uow(mut self, uow: UnitOfWork) -> Self {
        self.uow = Some(uow);
        self
    }

    pub async fn execute(&self, request: DeleteOrganizationRequest) -> Result<(), BusinessError> {
        <Self as AsyncDeleteEntityUseCase<DeleteOrganizationRequest, ()>>::execute(self, request)
            .await
    }

    async fn execute_transactional_pipeline(
        validated: Context<Validated, ValidatedContextData>,
        repo: &dyn OrganizationRepository,
    ) -> Result<(), BusinessError> {
        validated.delete(repo).await
    }
}

#[async_trait]
impl AsyncDeleteEntityUseCase<DeleteOrganizationRequest, ()> for DeleteOrganizationService {
    async fn execute(&self, request: DeleteOrganizationRequest) -> Result<(), BusinessError> {
        let validated = Context::<Initial, _>::new(InitialContextData { request }).validate()?;

        if let Some(uow) = &self.uow {
            uow.run_tx(|_tx| {
                Self::execute_transactional_pipeline(validated, self.repository.as_ref())
            })
            .await
        } else {
            Self::execute_transactional_pipeline(validated, self.repository.as_ref()).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, OrganizationId};
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};

    #[tokio::test]
    async fn test_delete_organization_service_full_flow() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Para Deletar", doc)
            .with_audit(AuditFields::created("admin"));

        repo.save(&org).await.unwrap();

        let service = DeleteOrganizationService::new(repo.clone());

        let delete_req = DeleteOrganizationRequest {
            organization_id: org_id,
            updated_by: "deleter_admin".to_string(),
        };

        let result = service.execute(delete_req).await;
        assert!(result.is_ok());

        let find_res = OrganizationRepository::find_by_id(&repo, *org_id).await;
        assert!(find_res.is_err());
        assert_eq!(
            find_res.unwrap_err(),
            OrganizationBusinessErrors::not_found()
        );
    }

    #[tokio::test]
    async fn test_delete_organization_service_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let service = DeleteOrganizationService::new(repo);

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
        let service = DeleteOrganizationService::new(repo);
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
