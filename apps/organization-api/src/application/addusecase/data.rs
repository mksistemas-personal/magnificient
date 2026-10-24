use crate::domain::{Organization, OrganizationId};
use shared::entities::{AuditFields, Document, DocumentType};

pub struct AddOrganizationRequest {
    pub name: String,
    pub document_value: String,
    pub document_type: DocumentType,
    pub created_by: String,
}

pub struct InitialContextData {
    pub request: AddOrganizationRequest,
}

pub struct ValidatedContextData {
    pub request: AddOrganizationRequest,
    pub organization_id: OrganizationId,
    pub document: Document,
    pub audit: AuditFields,
}

pub struct CreatedContextData {
    pub request: AddOrganizationRequest,
    pub organization: Organization,
}
