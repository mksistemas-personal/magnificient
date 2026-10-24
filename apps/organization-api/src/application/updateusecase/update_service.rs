use crate::application::OrganizationBusinessErrors;
use crate::application::updateusecase::data::*;
use crate::application::updateusecase::states::*;
use crate::domain::{Organization, OrganizationRepository};
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::BusinessError;
use shared::entities::use_case::context::Context;
use shared::entities::use_case::use_case_traits::AsyncUpdateEntityUseCase;
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
        if request.name.trim().is_empty() {
            return Err(OrganizationBusinessErrors::name_empty());
        }
        if request.updated_by.trim().is_empty() {
            return Err(OrganizationBusinessErrors::updater_required());
        }

        Ok(Context::new(ValidatedContextData { request }))
    }
}

trait LoadOrganizationContext {
    async fn load(
        self,
        repository: &dyn OrganizationRepository,
    ) -> Result<Context<Loaded, LoadedContextData>, BusinessError>;
}

impl LoadOrganizationContext for Context<Validated, ValidatedContextData> {
    async fn load(
        self,
        repository: &dyn OrganizationRepository,
    ) -> Result<Context<Loaded, LoadedContextData>, BusinessError> {
        let org_id = self.data.request.organization_id;
        let organization = repository.find_by_id(*org_id).await?;

        Ok(Context::new(LoadedContextData {
            request: self.data.request,
            organization,
        }))
    }
}

trait ChangeOrganizationContext {
    fn change(self) -> Result<Context<Changed, ChangedContextData>, BusinessError>;
}

impl ChangeOrganizationContext for Context<Loaded, LoadedContextData> {
    fn change(mut self) -> Result<Context<Changed, ChangedContextData>, BusinessError> {
        let name = self.data.request.name;
        let updated_by = self.data.request.updated_by;

        self.data.organization.update(name, updated_by);

        Ok(Context::new(ChangedContextData {
            organization: self.data.organization,
        }))
    }
}

trait PersistOrganizationContext {
    async fn save(
        self,
        repository: &dyn OrganizationRepository,
    ) -> Result<Organization, BusinessError>;
}

impl PersistOrganizationContext for Context<Changed, ChangedContextData> {
    async fn save(
        self,
        repository: &dyn OrganizationRepository,
    ) -> Result<Organization, BusinessError> {
        repository.save(&self.data.organization).await
    }
}

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

    pub async fn execute(
        &self,
        request: UpdateOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        <Self as AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>>::execute(
            self, request,
        )
        .await
    }

    async fn execute_transactional_pipeline(
        validated: Context<Validated, ValidatedContextData>,
        repo: &dyn OrganizationRepository,
    ) -> Result<Organization, BusinessError> {
        validated.load(repo).await?.change()?.save(repo).await
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
    use crate::domain::OrganizationId;
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
