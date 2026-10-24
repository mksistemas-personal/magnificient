use crate::api::organization_shared_dtos::default_user;
use crate::application::updateusecase::data::UpdateOrganizationRequest;
use crate::domain::OrganizationId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateOrganizationDto {
    pub name: String,
    #[serde(default = "default_user")]
    pub updated_by: String,
}

impl UpdateOrganizationDto {
    pub fn into_request(self, organization_id: OrganizationId) -> UpdateOrganizationRequest {
        UpdateOrganizationRequest {
            name: self.name,
            updated_by: self.updated_by,
            organization_id,
        }
    }
}

impl From<(OrganizationId, UpdateOrganizationDto)> for UpdateOrganizationRequest {
    fn from((organization_id, dto): (OrganizationId, UpdateOrganizationDto)) -> Self {
        dto.into_request(organization_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsid::TSID;

    #[test]
    fn test_dto_into_request() {
        let dto = UpdateOrganizationDto {
            name: "Empresa Teste".to_string(),
            updated_by: "changer".to_string(),
        };
        let org_id = OrganizationId::new(TSID::from(100u64));

        let req = dto.into_request(org_id);
        assert_eq!(req.name, "Empresa Teste");
        assert_eq!(req.updated_by, "changer");
        assert_eq!(req.organization_id, org_id);

        let dto2 = UpdateOrganizationDto {
            name: "Empresa 2".to_string(),
            updated_by: "user".to_string(),
        };
        let req2: UpdateOrganizationRequest = (org_id, dto2).into();
        assert_eq!(req2.name, "Empresa 2");
        assert_eq!(req2.organization_id, org_id);
    }
}
