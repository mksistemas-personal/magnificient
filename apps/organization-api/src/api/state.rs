use crate::application::add_organization::AddOrganizationUseCaseImpl;

use crate::config::organization_config::OrganizationAppConfig;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<OrganizationAppConfig>,
    pub add_organization_use_case: Arc<AddOrganizationUseCaseImpl>,
}

impl AppState {
    pub fn new(config: Arc<OrganizationAppConfig>) -> Self {
        Self {
            config,
            add_organization_use_case: Arc::new(AddOrganizationUseCaseImpl),
        }
    }
}
