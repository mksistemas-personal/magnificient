use crate::application::OrganizationBusinessErrors;
use crate::application::byidusecase::data::*;
use crate::application::byidusecase::states::*;
use crate::domain::{Organization, OrganizationRepository};
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::BusinessError;
use shared::entities::use_case::context::Context;
use shared::entities::use_case::use_case_traits::AsyncGetByIdUseCase;
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

        Ok(Context::new(ValidatedContextData { request }))
    }
}

trait FindOrganizationByIdContext {
    async fn find(
        self,
        repository: &dyn OrganizationRepository,
    ) -> Result<Organization, BusinessError>;
}

impl FindOrganizationByIdContext for Context<Validated, ValidatedContextData> {
    async fn find(
        self,
        repository: &dyn OrganizationRepository,
    ) -> Result<Organization, BusinessError> {
        let org_id = self.data.request.organization_id;
        repository.find_by_id(*org_id).await
    }
}

#[derive(Clone)]
pub struct FindOrganizationByIdService {
    repository: Arc<dyn OrganizationRepository>,
    uow: Option<UnitOfWork>,
}

impl FindOrganizationByIdService {
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

    pub async fn execute(
        &self,
        request: FindOrganizationByIdRequest,
    ) -> Result<Organization, BusinessError> {
        <Self as AsyncGetByIdUseCase<FindOrganizationByIdRequest, Organization>>::execute(
            self, request,
        )
        .await
    }

    async fn execute_pipeline(
        validated: Context<Validated, ValidatedContextData>,
        repo: &dyn OrganizationRepository,
    ) -> Result<Organization, BusinessError> {
        validated.find(repo).await
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
        let validated = Context::<Initial, _>::new(InitialContextData { request }).validate()?;
        Self::execute_pipeline(validated, self.repository.as_ref()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, OrganizationId};
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};

    #[tokio::test]
    async fn test_find_organization_by_id_service_full_flow() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Leitura", doc)
            .with_audit(AuditFields::created("admin"));

        repo.save(&org).await.unwrap();

        let service = FindOrganizationByIdService::new(repo.clone());
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
        let service = FindOrganizationByIdService::new(repo);

        let request = FindOrganizationByIdRequest::new(OrganizationId::new(TSID::from(999u64)));

        let result = service.execute(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_find_organization_by_id_validation_errors() {
        let repo = InMemoryOrganizationRepository::new();
        let service = FindOrganizationByIdService::new(repo);

        let invalid_req = FindOrganizationByIdRequest::new(OrganizationId::new(TSID::from(0u64)));
        let result = service.execute(invalid_req).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            OrganizationBusinessErrors::id_invalid()
        );
    }
}
