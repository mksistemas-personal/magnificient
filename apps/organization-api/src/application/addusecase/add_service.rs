use crate::application::OrganizationBusinessErrors;
use crate::application::addusecase::data::{
    AddOrganizationRequest, CreatedContextData, InitialContextData, ValidatedContextData,
};
use crate::application::addusecase::states::{Builded, Initial, Validated};
use crate::domain::{Organization, OrganizationId, OrganizationRepository};
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::use_case::context::Context;
use shared::entities::{AsyncAddEntityUseCase, AuditFields, BusinessError, Document};
use std::sync::Arc;
use tsid::create_tsid;

trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}
impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        let organization_id = OrganizationId::new(create_tsid());

        let name = request.name.clone();
        if name.trim().is_empty() {
            Err(OrganizationBusinessErrors::name_empty())?
        };

        let document =
            Document::new_validated(&request.document_value, request.document_type.clone())
                .map_err(|_| OrganizationBusinessErrors::document_invalid())?;

        let created_by = request.created_by.clone();
        if created_by.trim().is_empty() {
            Err(OrganizationBusinessErrors::creator_required())?;
        }

        let audit = AuditFields::created(&request.created_by);

        Ok(Context::<Validated, ValidatedContextData>::new(
            ValidatedContextData {
                request,
                organization_id,
                document,
                audit,
            },
        ))
    }
}

trait BuildOrganizationContext {
    fn build(self) -> Result<Context<Builded, CreatedContextData>, BusinessError>;
}

impl BuildOrganizationContext for Context<Validated, ValidatedContextData> {
    fn build(self) -> Result<Context<Builded, CreatedContextData>, BusinessError> {
        let request = self.data.request;
        let organization = Organization::new(
            self.data.organization_id,
            request.name.clone(),
            self.data.document.clone(),
        )
        .with_audit(self.data.audit.clone());
        Ok(Context::<Builded, CreatedContextData>::new(
            CreatedContextData {
                request,
                organization,
            },
        ))
    }
}

trait PersistOrganizationContext {
    async fn save(
        self,
        repository: Arc<dyn OrganizationRepository>,
        uow: Option<UnitOfWork>,
    ) -> Result<Organization, BusinessError>;
}

impl PersistOrganizationContext for Context<Builded, CreatedContextData> {
    async fn save(
        self,
        repository: Arc<dyn OrganizationRepository>,
        uow: Option<UnitOfWork>,
    ) -> Result<Organization, BusinessError> {
        if let Some(uow) = uow {
            uow.run_tx(|_tx| async { repository.save(&self.data.organization).await })
                .await
        } else {
            repository.save(&self.data.organization).await
        }
    }
}

#[derive(Clone)]
pub struct AddOrganizationService {
    repository: Arc<dyn OrganizationRepository>,
    uow: Option<UnitOfWork>,
}

impl AddOrganizationService {
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

    /// Cria uma nova instância do caso de uso a partir de um `Arc<dyn OrganizationRepository>` e `UnitOfWork`.
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
        request: AddOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        <Self as AsyncAddEntityUseCase<AddOrganizationRequest, Organization>>::execute(
            self, request,
        )
        .await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<AddOrganizationRequest, Organization> for AddOrganizationService {
    async fn execute(
        &self,
        request: AddOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .build()?
            .save(self.repository.clone(), self.uow.clone())
            .await
    }
}
