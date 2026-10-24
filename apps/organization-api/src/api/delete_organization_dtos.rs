use crate::api::organization_shared_dtos::default_user;
use crate::application::deleteusecase::data::DeleteOrganizationRequest;
use crate::domain::OrganizationId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteOrganizationParams {
    #[serde(alias = "updated_by", default = "default_user")]
    pub update_by: String,
}

impl DeleteOrganizationParams {
    pub fn into_request(self, organization_id: OrganizationId) -> DeleteOrganizationRequest {
        DeleteOrganizationRequest {
            organization_id,
            updated_by: self.update_by,
        }
    }
}

impl From<(OrganizationId, DeleteOrganizationParams)> for DeleteOrganizationRequest {
    fn from((organization_id, params): (OrganizationId, DeleteOrganizationParams)) -> Self {
        params.into_request(organization_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsid::TSID;

    #[test]
    fn test_delete_params_into_request() {
        let params = DeleteOrganizationParams {
            update_by: "deleter".to_string(),
        };
        let org_id = OrganizationId::new(TSID::from(100u64));

        let req = params.into_request(org_id);
        assert_eq!(req.updated_by, "deleter");
        assert_eq!(req.organization_id, org_id);

        let params2 = DeleteOrganizationParams {
            update_by: "admin".to_string(),
        };
        let req2: DeleteOrganizationRequest = (org_id, params2).into();
        assert_eq!(req2.updated_by, "admin");
        assert_eq!(req2.organization_id, org_id);
    }
}
