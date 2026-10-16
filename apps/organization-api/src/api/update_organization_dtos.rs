use crate::application::UpdateOrganizationRequest;
use serde::{Deserialize, Serialize};
use shared::errors::api_error::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateOrganizationDto {
    pub name: String,
    pub updated_by: String,
}

impl TryFrom<UpdateOrganizationDto> for UpdateOrganizationRequest {
    type Error = ApiError;

    fn try_from(dto: UpdateOrganizationDto) -> Result<Self, Self::Error> {
        Ok(UpdateOrganizationRequest {
            name: dto.name,
            updated_by: dto.updated_by,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dto_try_into_request() {
        let dto = UpdateOrganizationDto {
            name: "Empresa Teste".to_string(),
            updated_by: "changer".to_string(),
        };

        let req: Result<UpdateOrganizationRequest, _> = dto.try_into();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.name, "Empresa Teste");
    }
}
