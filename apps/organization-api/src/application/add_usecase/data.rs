use crate::domain::{Organization, OrganizationId};
use shared::entities::{Address, AuditFields, Document, DocumentType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddOrganizationRequest {
    pub name: String,
    pub document_value: String,
    pub document_type: DocumentType,
    pub address: Option<Address>,
    pub created_by: String,
}

impl AddOrganizationRequest {
    pub fn new(
        name: impl Into<String>,
        document_value: impl Into<String>,
        document_type: DocumentType,
        created_by: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            document_value: document_value.into(),
            document_type,
            address: None,
            created_by: created_by.into(),
        }
    }

    pub fn with_address(mut self, address: Address) -> Self {
        self.address = Some(address);
        self
    }
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

pub struct AddressValidatedContextData {
    pub request: AddOrganizationRequest,
    pub organization_id: OrganizationId,
    pub document: Document,
    pub address: Option<Address>,
    pub audit: AuditFields,
}

pub struct CreatedContextData {
    pub organization: Organization,
}
