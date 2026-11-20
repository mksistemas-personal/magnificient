use crate::domain::OrganizationId;

#[derive(Debug, Clone)]
pub struct DeleteOrganizationSuppliersRequest {
    pub organization_id: OrganizationId,
    pub updated_by: String,
}
