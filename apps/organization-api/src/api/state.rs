use crate::application::add_organization_service::AddOrganizationServiceImpl;
use crate::config::organization_config::OrganizationAppConfig;
use crate::domain::organization_repository::OrganizationRepository;
use crate::infrastructure::postgres_organization_repository::PostgresOrganizationRepository;
use database::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<OrganizationAppConfig>,
    pub db: DatabaseConnection,
    pub add_organization_use_case: Arc<AddOrganizationServiceImpl>,
}

impl AppState {
    pub fn new(config: Arc<OrganizationAppConfig>, db: DatabaseConnection) -> Self {
        let repository = PostgresOrganizationRepository::from_connection(&db);
        Self {
            config,
            db,
            add_organization_use_case: Arc::new(AddOrganizationServiceImpl::new(repository)),
        }
    }

    pub fn new_with_repository(
        config: Arc<OrganizationAppConfig>,
        db: DatabaseConnection,
        repository: impl OrganizationRepository + 'static,
    ) -> Self {
        Self {
            config,
            db,
            add_organization_use_case: Arc::new(AddOrganizationServiceImpl::new(repository)),
        }
    }

    pub fn new_with_use_case(
        config: Arc<OrganizationAppConfig>,
        db: DatabaseConnection,
        use_case: Arc<AddOrganizationServiceImpl>,
    ) -> Self {
        Self {
            config,
            db,
            add_organization_use_case: use_case,
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn config(&self) -> &OrganizationAppConfig {
        &self.config
    }
}
