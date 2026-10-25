use shared::types::page::PageRequest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindAllOrganizationsRequest {
    pub name: Option<String>,
    pub page_request: PageRequest,
}

impl FindAllOrganizationsRequest {
    pub fn new(name: Option<String>, page_request: PageRequest) -> Self {
        Self { name, page_request }
    }

    pub fn with_name(name: impl Into<String>, page_request: PageRequest) -> Self {
        Self {
            name: Some(name.into()),
            page_request,
        }
    }

    pub fn without_filter(page_request: PageRequest) -> Self {
        Self {
            name: None,
            page_request,
        }
    }

    pub fn from_page(page: u32, page_size: u32) -> Self {
        Self {
            name: None,
            page_request: PageRequest::new(page, page_size),
        }
    }

    pub fn from_name_and_page(name: impl Into<String>, page: u32, page_size: u32) -> Self {
        Self {
            name: Some(name.into()),
            page_request: PageRequest::new(page, page_size),
        }
    }
}

impl From<(Option<String>, PageRequest)> for FindAllOrganizationsRequest {
    fn from((name, page_request): (Option<String>, PageRequest)) -> Self {
        Self { name, page_request }
    }
}

impl From<(String, PageRequest)> for FindAllOrganizationsRequest {
    fn from((name, page_request): (String, PageRequest)) -> Self {
        Self {
            name: Some(name),
            page_request,
        }
    }
}

impl From<(&str, PageRequest)> for FindAllOrganizationsRequest {
    fn from((name, page_request): (&str, PageRequest)) -> Self {
        Self {
            name: Some(name.to_string()),
            page_request,
        }
    }
}

impl From<PageRequest> for FindAllOrganizationsRequest {
    fn from(page_request: PageRequest) -> Self {
        Self {
            name: None,
            page_request,
        }
    }
}

pub type GetAllOrganizationsRequest = FindAllOrganizationsRequest;
pub type FindOrganizationsByNameRequest = FindAllOrganizationsRequest;

pub struct InitialContextData {
    pub request: FindAllOrganizationsRequest,
}

pub struct ValidatedContextData {
    pub request: FindAllOrganizationsRequest,
}
