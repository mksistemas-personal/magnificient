use crate::domain::OrganizationId;

#[derive(Debug, Clone)]
pub struct DeleteOrganizationProductsRequest {
    pub organization_id: OrganizationId,
    pub updated_by: String,
}
