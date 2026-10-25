use crate::application::allusecase::data::FindAllOrganizationsRequest;
use serde::{Deserialize, Serialize};
use shared::types::page::PageRequest;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FindAllOrganizationsParams {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default, alias = "pageSize", alias = "limit")]
    pub page_size: Option<u32>,
}

impl FindAllOrganizationsParams {
    pub fn new(name: Option<String>, page: Option<u32>, page_size: Option<u32>) -> Self {
        Self {
            name,
            page,
            page_size,
        }
    }

    pub fn into_request(self) -> FindAllOrganizationsRequest {
        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(20);
        FindAllOrganizationsRequest::new(self.name, PageRequest::new(page, page_size))
    }
}

impl From<FindAllOrganizationsParams> for FindAllOrganizationsRequest {
    fn from(params: FindAllOrganizationsParams) -> Self {
        params.into_request()
    }
}

pub type GetAllOrganizationsParams = FindAllOrganizationsParams;
pub type FindOrganizationsQuery = FindAllOrganizationsParams;
pub type FindAllOrganizationsQuery = FindAllOrganizationsParams;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_all_params_defaults() {
        let params = FindAllOrganizationsParams::default();
        let req: FindAllOrganizationsRequest = params.into();
        assert_eq!(req.name, None);
        assert_eq!(req.page_request.page, 1);
        assert_eq!(req.page_request.page_size, 20);
    }

    #[test]
    fn test_find_all_params_custom() {
        let params = FindAllOrganizationsParams {
            name: Some("Acme".to_string()),
            page: Some(3),
            page_size: Some(15),
        };
        let req = params.into_request();
        assert_eq!(req.name, Some("Acme".to_string()));
        assert_eq!(req.page_request.page, 3);
        assert_eq!(req.page_request.page_size, 15);
    }
}
