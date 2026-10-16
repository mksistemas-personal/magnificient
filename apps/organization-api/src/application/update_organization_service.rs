use crate::application::OrganizationBusinessErrors;
use crate::domain::{Organization, OrganizationRepository};
use database::UnitOfWork;
use shared::entities::BusinessError;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOrganizationRequest {
    pub name: String,
    pub updated_by: String,
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
    pub fn new_with_uow(
        repository: impl OrganizationRepository + 'static,
        uow: UnitOfWork,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            uow: Some(uow),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::organization::{Organization, OrganizationId};
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
        let context = UpdateOrganizationContext {
            update_request: UpdateOrganizationRequest {
                name: "Novo Nome".to_string(),
                updated_by: "editor".to_string(),
            },
            organization: None,
        };

        let result = UpdateOrganizationServiceImpl::change_organization(context);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }
}

