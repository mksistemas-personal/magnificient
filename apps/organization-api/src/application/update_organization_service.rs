use crate::application::OrganizationBusinessErrors;
use crate::domain::{Organization, OrganizationId, OrganizationRepository};
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::use_case::use_case_traits::AsyncUpdateEntityUseCase;
use shared::entities::{AsyncResultExt, BusinessError};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOrganizationRequest {
    pub name: String,
    pub updated_by: String,
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UpdateOrganizationContext {
    update_request: UpdateOrganizationRequest,
    pub organization: Option<Organization>,
}

impl UpdateOrganizationContext {
    pub fn name(&self) -> &str {
        &self.update_request.name
    }

    pub fn updated_by(&self) -> &str {
        &self.update_request.updated_by
    }

    pub fn organization_id(&self) -> &OrganizationId {
        &self.update_request.organization_id
    }
}

impl From<UpdateOrganizationRequest> for UpdateOrganizationContext {
    fn from(request: UpdateOrganizationRequest) -> Self {
        Self {
            update_request: request,
            organization: None,
        }
    }
}

#[derive(Clone)]
pub struct UpdateOrganizationServiceImpl {
    repository: Arc<dyn OrganizationRepository>,
    uow: Option<UnitOfWork>,
}

impl UpdateOrganizationServiceImpl {
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

    fn validate_organization_id(context: UpdateOrganizationContext) -> Result<UpdateOrganizationContext, BusinessError> {
        if !context.organization_id().is_valid() {
            Err(OrganizationBusinessErrors::id_invalid())?
        };
        Ok(context)
    }

    fn validate_name(context: UpdateOrganizationContext) -> Result<UpdateOrganizationContext, BusinessError> {
        if context.name().trim().is_empty() {
            Err(OrganizationBusinessErrors::name_empty())?
        };
        Ok(context)
    }

    fn validate_updater(context: UpdateOrganizationContext) -> Result<UpdateOrganizationContext, BusinessError> {
        if context.updated_by().trim().is_empty() {
            Err(OrganizationBusinessErrors::updater_required())?
        };
        Ok(context)
    }

    fn change_organization(mut context: UpdateOrganizationContext) -> Result<UpdateOrganizationContext, BusinessError> {
        let organization = context
            .organization
            .as_mut()
            .ok_or_else(OrganizationBusinessErrors::not_found)?;

        let name = std::mem::take(&mut context.update_request.name);
        let updated_by = std::mem::take(&mut context.update_request.updated_by);
        organization.update(name, updated_by);

        Ok(context)
    }

    async fn save_organization(&self, context: UpdateOrganizationContext) -> Result<Organization, BusinessError> {
        let update_and_persist = |mut ctx: UpdateOrganizationContext| async move {
            let org = self.repository.find_by_id(**ctx.organization_id()).await?;
            ctx.organization = Some(org);
            let ctx = Self::change_organization(ctx)?;
            let organization = ctx
                .organization
                .as_ref()
                .ok_or_else(OrganizationBusinessErrors::not_found)?;
            self.repository.save(organization).await
        };

        if let Some(uow) = &self.uow {
            uow.run_tx(|_tx| update_and_persist(context)).await
        } else {
            update_and_persist(context).await
        }
    }

    /// Executa o caso de uso encadeando as etapas de validação, leitura, atualização e persistência.
    pub async fn execute(&self, request: UpdateOrganizationRequest) -> Result<Organization, BusinessError> {
        <Self as AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>>::execute(self, request).await
    }
}

#[async_trait]
impl AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization> for UpdateOrganizationServiceImpl {
    async fn execute(&self, request: UpdateOrganizationRequest) -> Result<Organization, BusinessError> {
        Self::validate_organization_id(request.into())
            .and_then(Self::validate_name)
            .and_then(Self::validate_updater)
            .and_then_async(|context| async move {
                self.save_organization(context).await
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::organization::{Organization, OrganizationId};
    use crate::infrastructure::in_memory_organization_repository::InMemoryOrganizationRepository;
    use shared::entities::audit::audit_fields::AuditFields;
    use shared::entities::{Document, DocumentType, TSID};

    #[test]
    fn test_change_organization_success() {
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Nome Antigo", doc)
            .with_audit(AuditFields::created("admin"));

        let context = UpdateOrganizationContext {
            update_request: UpdateOrganizationRequest {
                name: "Novo Nome".to_string(),
                updated_by: "editor".to_string(),
                organization_id: org_id,
            },
            organization: Some(org),
        };

        let result = UpdateOrganizationServiceImpl::change_organization(context);
        assert!(result.is_ok());

        let context = result.unwrap();
        let org = context.organization.as_ref().unwrap();
        assert_eq!(org.name(), "Novo Nome");
        assert_eq!(org.audit.as_ref().unwrap().created_by(), "admin");
        assert_eq!(org.audit.as_ref().unwrap().updated_by(), "editor");
    }

    #[test]
    fn test_change_organization_not_found() {
        let org_id = OrganizationId::new(TSID::from(100u64));
        let context = UpdateOrganizationContext {
            update_request: UpdateOrganizationRequest {
                name: "Novo Nome".to_string(),
                updated_by: "editor".to_string(),
                organization_id: org_id,
            },
            organization: None,
        };

        let result = UpdateOrganizationServiceImpl::change_organization(context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_save_organization_success() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(200u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Nome Original", doc)
            .with_audit(AuditFields::created("criador"));

        repo.save(&org).await.unwrap();

        let service = UpdateOrganizationServiceImpl::new(repo.clone());
        let context = UpdateOrganizationContext {
            update_request: UpdateOrganizationRequest {
                name: "Nome Atualizado".to_string(),
                updated_by: "modificador".to_string(),
                organization_id: org_id,
            },
            organization: None,
        };

        let updated_org = service.save_organization(context).await.unwrap();
        assert_eq!(updated_org.name(), "Nome Atualizado");
        assert_eq!(updated_org.audit.as_ref().unwrap().updated_by(), "modificador");

        // Confirma persistência no repositório
        let persisted = repo.find_by_id(&org_id).unwrap();
        assert_eq!(persisted.name(), "Nome Atualizado");
        assert_eq!(persisted.audit.as_ref().unwrap().updated_by(), "modificador");
    }

    #[tokio::test]
    async fn test_save_organization_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(404u64));
        let service = UpdateOrganizationServiceImpl::new(repo);

        let context = UpdateOrganizationContext {
            update_request: UpdateOrganizationRequest {
                name: "Qualquer".to_string(),
                updated_by: "modificador".to_string(),
                organization_id: org_id,
            },
            organization: None,
        };

        let result = service.save_organization(context).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_execute_update_organization_full_flow() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(300u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Antes", doc)
            .with_audit(AuditFields::created("admin"));

        repo.save(&org).await.unwrap();

        let service = UpdateOrganizationServiceImpl::new(repo.clone());
        let req = UpdateOrganizationRequest {
            organization_id: org_id,
            name: "Depois".to_string(),
            updated_by: "admin2".to_string(),
        };

        let res = service.execute(req).await;
        assert!(res.is_ok());
        let res_org = res.unwrap();
        assert_eq!(res_org.name(), "Depois");
        assert_eq!(res_org.audit.as_ref().unwrap().updated_by(), "admin2");

        // Validação de erro de nome vazio
        let invalid_name_req = UpdateOrganizationRequest {
            organization_id: org_id,
            name: "".to_string(),
            updated_by: "admin2".to_string(),
        };
        assert_eq!(
            service.execute(invalid_name_req).await.unwrap_err(),
            OrganizationBusinessErrors::name_empty()
        );

        // Validação de erro de updater vazio
        let invalid_updater_req = UpdateOrganizationRequest {
            organization_id: org_id,
            name: "Valido".to_string(),
            updated_by: "".to_string(),
        };
        assert_eq!(
            service.execute(invalid_updater_req).await.unwrap_err(),
            OrganizationBusinessErrors::updater_required()
        );
    }
}

