use crate::application::errors::OrganizationBusinessErrors;
use crate::domain::organization::{Organization, OrganizationId};
use crate::domain::organization_repository::OrganizationRepository;
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::{AsyncAddEntityUseCase, AsyncResultExt, AuditFields, BusinessError, Document, DocumentType};
use std::sync::Arc;
use tsid::create_tsid;

pub struct AddOrganizationRequest {
    pub name: String,
    pub document_value: String,
    pub document_type: DocumentType,
    pub created_by: String,
}

pub struct AddOrganizationContext {
    add_request: AddOrganizationRequest,
    pub organization_id: Option<OrganizationId>,
    pub document: Option<Document>,
    pub organization: Option<Organization>,
}

impl AddOrganizationContext {
    pub fn name(&self) -> &str {
        &self.add_request.name
    }
    pub fn document_value(&self) -> &str {
        &self.add_request.document_value
    }
    pub fn document_type(&self) -> &DocumentType {
        &self.add_request.document_type
    }
    pub fn created_by(&self) -> &str {
        &self.add_request.created_by
    }
}

impl From<AddOrganizationRequest> for AddOrganizationContext {
    fn from(request: AddOrganizationRequest) -> Self {
        Self {
            add_request: request,
            organization_id: None,
            document: None,
            organization: None,
        }
    }
}

#[derive(Clone)]
pub struct AddOrganizationServiceImpl {
    repository: Arc<dyn OrganizationRepository>,
    uow: Option<UnitOfWork>,
}

impl AddOrganizationServiceImpl {
    /// Cria uma nova instância do caso de uso com o repositório fornecido.
    pub fn new(repository: impl OrganizationRepository + 'static) -> Self {
        Self {
            repository: Arc::new(repository),
            uow: None,
        }
    }

    /// Cria uma nova instância do caso de uso com o repositório e a Unidade de Trabalho (`UnitOfWork`).
    pub fn new_with_uow(
        repository: impl OrganizationRepository + 'static,
        uow: UnitOfWork,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            uow: Some(uow),
        }
    }

    /// Cria uma nova instância do caso de uso a partir de um `Arc<dyn OrganizationRepository>`.
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

    /// Define ou atualiza a Unidade de Trabalho (`UnitOfWork`) para persistência transacional.
    pub fn with_uow(mut self, uow: UnitOfWork) -> Self {
        self.uow = Some(uow);
        self
    }

    /// Valida e cria o identificador da organização (Railway Step)
    fn validate_id(mut context: AddOrganizationContext) -> Result<AddOrganizationContext, BusinessError> {
        context.organization_id = Some(OrganizationId::new(create_tsid()));
        Ok(context)
    }

    /// Valida o nome da organização (Railway Step)
    fn validate_name(context: AddOrganizationContext) -> Result<AddOrganizationContext, BusinessError> {
        if context.name().trim().is_empty() {
            Err(OrganizationBusinessErrors::name_empty())?
        };
        Ok(context)
    }

    /// Valida e cria o documento da organização (Railway Step)
    fn validate_document(mut context: AddOrganizationContext) -> Result<AddOrganizationContext, BusinessError> {
        context.document = Some(
            Document::new_validated(
                context.document_value(),
                context.document_type().clone(),
            )
                .map_err(|_| OrganizationBusinessErrors::document_invalid())?,
        );
        Ok(context)
    }

    /// Valida o autor da criação da organização (Railway Step)
    fn validate_creator(context: AddOrganizationContext) -> Result<AddOrganizationContext, BusinessError> {
        if context.created_by().trim().is_empty() {
            Err(OrganizationBusinessErrors::creator_required())?
        };
        Ok(context)
    }

    /// Monta a entidade Organization com auditoria a partir dos dados validados (Railway Step)
    fn build_organization(mut context: AddOrganizationContext) -> Result<AddOrganizationContext, BusinessError> {
        let id = context.organization_id.ok_or_else(OrganizationBusinessErrors::id_invalid)?;
        let document = context.document.take().ok_or_else(OrganizationBusinessErrors::document_invalid)?;
        let created_by = std::mem::take(&mut context.add_request.created_by);
        let name = std::mem::take(&mut context.add_request.name);
        let audit = AuditFields::created(created_by);
        context.organization = Some(Organization::new(id, name, document).with_audit(audit));
        Ok(context)
    }

    /// Salva a organização no banco de dados através do repositório (com suporte atômico transacional via UnitOfWork)
    async fn save_organization(&self, organization: &Organization) -> Result<Organization, BusinessError> {
        if let Some(uow) = &self.uow {
            uow.run_tx(|_tx| async {
                self.repository.save(organization).await
            })
            .await
        } else {
            self.repository.save(organization).await
        }
    }

    /// Executa o caso de uso encadeando as etapas de validação, construção e persistência atômica no Railway.
    pub async fn execute(&self, request: AddOrganizationRequest) -> Result<Organization, BusinessError> {
        <Self as AsyncAddEntityUseCase<AddOrganizationRequest, Organization>>::execute(self, request).await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<AddOrganizationRequest, Organization> for AddOrganizationServiceImpl {
    async fn execute(&self, request: AddOrganizationRequest) -> Result<Organization, BusinessError> {
        Self::validate_id(request.into())
            .and_then(Self::validate_name)
            .and_then(Self::validate_document)
            .and_then(Self::validate_creator)
            .and_then(Self::build_organization)
            .and_then_async(|context| async move {
                let organization = context
                    .organization
                    .ok_or_else(OrganizationBusinessErrors::not_found)?;

                self.save_organization(&organization).await
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::in_memory_organization_repository::InMemoryOrganizationRepository;

    #[tokio::test]
    async fn test_add_organization_use_case_success() {
        let repository = InMemoryOrganizationRepository::new();
        let use_case = AddOrganizationServiceImpl::new(repository.clone());
        let request = AddOrganizationRequest {
            name: "Empresa XPTO".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "admin_user".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_ok());

        let org = result.unwrap();
        assert!(org.id().is_valid());
        assert_eq!(org.name(), "Empresa XPTO");
        assert_eq!(org.document().value(), "00.000.000/0001-91");
        assert!(org.audit.is_some());
        assert_eq!(org.audit.as_ref().unwrap().created_by(), "admin_user");

        // Verifica se a entidade foi de fato persistida no repositório
        assert_eq!(repository.len(), 1);
        let saved_org = repository.find_by_id(&org.id()).expect("Organização deveria estar no repositório");
        assert_eq!(saved_org.name(), "Empresa XPTO");
    }

    #[tokio::test]
    async fn test_add_organization_use_case_duplicate_document() {
        let repository = InMemoryOrganizationRepository::new();
        let use_case = AddOrganizationServiceImpl::new(repository.clone());

        let request1 = AddOrganizationRequest {
            name: "Empresa XPTO 1".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "admin_user".to_string(),
        };
        let request2 = AddOrganizationRequest {
            name: "Empresa XPTO 2".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "admin_user".to_string(),
        };

        let result1 = use_case.execute(request1).await;
        assert!(result1.is_ok());

        let result2 = use_case.execute(request2).await;
        assert!(result2.is_err());
        assert_eq!(result2.unwrap_err(), OrganizationBusinessErrors::document_already_in_use());
    }

    #[tokio::test]
    async fn test_add_organization_use_case_invalid_doc() {
        let repository = InMemoryOrganizationRepository::new();
        let use_case = AddOrganizationServiceImpl::new(repository);
        let request = AddOrganizationRequest {
            name: "Empresa XPTO".to_string(),
            document_value: "111.111.111-11".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "admin_user".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BusinessError::InvalidData(_)));
        assert!(err.message().contains(OrganizationBusinessErrors::ORG_DOCUMENT_INVALID));
    }

    #[tokio::test]
    async fn test_add_organization_use_case_empty_name() {
        let repository = InMemoryOrganizationRepository::new();
        let use_case = AddOrganizationServiceImpl::new(repository);
        let request = AddOrganizationRequest {
            name: "".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "admin_user".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, OrganizationBusinessErrors::name_empty());
    }

    #[tokio::test]
    async fn test_add_organization_use_case_empty_creator() {
        let repository = InMemoryOrganizationRepository::new();
        let use_case = AddOrganizationServiceImpl::new(repository);
        let request = AddOrganizationRequest {
            name: "Empresa XPTO".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, OrganizationBusinessErrors::creator_required());
    }

    #[test]
    fn test_add_organization_context_getters() {
        let request = AddOrganizationRequest {
            name: "Empresa Teste".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "autor".to_string(),
        };

        let context: AddOrganizationContext = request.into();
        assert_eq!(context.name(), "Empresa Teste");
        assert_eq!(context.document_value(), "00.000.000/0001-91");
        assert_eq!(*context.document_type(), DocumentType::Cnpj);
        assert_eq!(context.created_by(), "autor");
    }

    #[tokio::test]
    async fn test_add_organization_service_constructors() {
        let repository = InMemoryOrganizationRepository::new();
        let service = AddOrganizationServiceImpl::new(repository.clone());
        assert!(service.uow.is_none());

        let arc_repo = Arc::new(repository);
        let service_arc = AddOrganizationServiceImpl::new_with_arc(arc_repo.clone());
        assert!(service_arc.uow.is_none());
    }
}
