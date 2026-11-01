use crate::domain::{Organization, OrganizationId};
use shared::entities::Address;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOrganizationRequest {
    pub organization_id: OrganizationId,
    pub name: String,
    pub address: Option<Address>,
    pub updated_by: String,
}

impl UpdateOrganizationRequest {
    pub fn new(
        organization_id: OrganizationId,
        name: impl Into<String>,
        updated_by: impl Into<String>,
    ) -> Self {
        Self {
            organization_id,
            name: name.into(),
            address: None,
            updated_by: updated_by.into(),
        }
    }

    pub fn with_address(mut self, address: Address) -> Self {
        self.address = Some(address);
        self
    }
}

pub struct InitialContextData {
    pub request: UpdateOrganizationRequest,
}

pub struct ValidatedContextData {
    pub request: UpdateOrganizationRequest,
}

pub struct AddressValidatedContextData {
    pub request: UpdateOrganizationRequest,
    pub address: Option<Address>,
}

pub struct LoadedContextData {
    pub request: UpdateOrganizationRequest,
    pub address: Option<Address>,
    pub organization: Organization,
}

pub struct ChangedContextData {
    pub organization: Organization,
}
