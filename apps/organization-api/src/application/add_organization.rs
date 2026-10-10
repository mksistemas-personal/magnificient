use crate::application::errors::OrganizationBusinessErrors;
use crate::domain::organization::{Organization, OrganizationId};
use shared::entities::{AddEntityUseCase, AuditFields, BusinessError, Document, DocumentType};

pub struct AddOrganizationRequest {
    pub id: i64,
    pub name: String,
    pub document_value: String,
    pub document_type: DocumentType,
    pub created_by: String,
}


struct AddOrganizationContext {
    pub add_request: AddOrganizationRequest,
    pub organization_id: Option<OrganizationId>,
    pub document: Option<Document>,
    pub organization: Option<Organization>,
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

pub struct AddOrganizationUseCaseImpl;

impl AddOrganizationUseCaseImpl {
    /// Valida e cria o identificador da organização (Railway Step)
    fn validate_id(mut context: AddOrganizationContext) -> Result<AddOrganizationContext, BusinessError> {
        context.organization_id = Some(
            OrganizationId::new_validated(context.add_request.id)
                .map_err(|_| OrganizationBusinessErrors::id_invalid())?,
        );
        Ok(context)
    }

    /// Valida o nome da organização (Railway Step)
    fn validate_name(mut context: AddOrganizationContext) -> Result<AddOrganizationContext, BusinessError> {
        if context.add_request.name.trim().is_empty() {
            Err(OrganizationBusinessErrors::name_empty())?
        };
        Ok(context)
    }

    /// Valida e cria o documento da organização (Railway Step)
    fn validate_document(mut context: AddOrganizationContext) -> Result<AddOrganizationContext, BusinessError> {
        context.document = Some(
            Document::new_validated(
                context.add_request.document_value.clone(),
                context.add_request.document_type.clone())
                .map_err(|_| OrganizationBusinessErrors::document_invalid())?,
        );
        Ok(context)
    }

    /// Valida o autor da criação da organização (Railway Step)
    fn validate_creator(mut context: AddOrganizationContext) -> Result<AddOrganizationContext, BusinessError> {
        if context.add_request.created_by.trim().is_empty() {
            Err(OrganizationBusinessErrors::creator_required())?
        };
        Ok(context)
    }

    /// Monta a entidade Organization com auditoria a partir dos dados validados
    fn build_organization(mut context: AddOrganizationContext) -> Result<AddOrganizationContext, BusinessError> {
        let id = context.organization_id.ok_or_else(OrganizationBusinessErrors::id_invalid)?;
        let document = context.document.take().ok_or_else(OrganizationBusinessErrors::document_invalid)?;
        let created_by = std::mem::take(&mut context.add_request.created_by);
        let name = std::mem::take(&mut context.add_request.name);
        let audit = AuditFields::created(created_by);
        context.organization = Some(Organization::new(id, name, document).with_audit(audit));
        Ok(context)
    }
}

impl AddEntityUseCase<AddOrganizationRequest, Result<Organization, BusinessError>> for AddOrganizationUseCaseImpl {
    /// Executa o caso de uso encadeando as etapas de validação e criação pelo padrão Railway Oriented Programming (ROP).
    fn execute(&self, request: AddOrganizationRequest) -> Result<Organization, BusinessError> {
        Self::validate_id(request.into())
            .and_then(Self::validate_name)
            .and_then(Self::validate_document)
            .and_then(Self::validate_creator)
            .and_then(Self::build_organization)
            .map(|context| context.organization.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_organization_use_case_success() {
        let use_case = AddOrganizationUseCaseImpl;
        let request = AddOrganizationRequest {
            id: 1,
            name: "Empresa XPTO".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "admin_user".to_string(),
        };

        let result = use_case.execute(request);
        assert!(result.is_ok());

        let org = result.unwrap();
        assert_eq!(org.id().value(), 1);
        assert_eq!(org.name(), "Empresa XPTO");
        assert_eq!(org.document().value(), "00.000.000/0001-91");
        assert!(org.audit.is_some());
        assert_eq!(org.audit.unwrap().created_by(), "admin_user");
    }

    #[test]
    fn test_add_organization_use_case_invalid_doc() {
        let use_case = AddOrganizationUseCaseImpl;
        let request = AddOrganizationRequest {
            id: 1,
            name: "Empresa XPTO".to_string(),
            document_value: "111.111.111-11".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "admin_user".to_string(),
        };

        let result = use_case.execute(request);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BusinessError::InvalidData(_)));
        assert!(err.message().contains(OrganizationBusinessErrors::ORG_DOCUMENT_INVALID));
    }

    #[test]
    fn test_add_organization_use_case_empty_name() {
        let use_case = AddOrganizationUseCaseImpl;
        let request = AddOrganizationRequest {
            id: 1,
            name: "".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "admin_user".to_string(),
        };

        let result = use_case.execute(request);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, OrganizationBusinessErrors::name_empty());
    }

    #[test]
    fn test_add_organization_use_case_invalid_id() {
        let use_case = AddOrganizationUseCaseImpl;
        let request = AddOrganizationRequest {
            id: -5,
            name: "Empresa XPTO".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "admin_user".to_string(),
        };

        let result = use_case.execute(request);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BusinessError::InvalidData(_)));
        assert!(err.message().contains(OrganizationBusinessErrors::ORG_ID_INVALID));
    }

    #[test]
    fn test_add_organization_use_case_empty_creator() {
        let use_case = AddOrganizationUseCaseImpl;
        let request = AddOrganizationRequest {
            id: 1,
            name: "Empresa XPTO".to_string(),
            document_value: "00.000.000/0001-91".to_string(),
            document_type: DocumentType::Cnpj,
            created_by: "".to_string(),
        };

        let result = use_case.execute(request);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, OrganizationBusinessErrors::creator_required());
    }
}
