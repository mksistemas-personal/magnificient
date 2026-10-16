use crate::application::add_organization_service::{AddOrganizationRequest, AddOrganizationServiceImpl};
use crate::application::{UpdateOrganizationRequest, UpdateOrganizationServiceImpl};
use crate::config::organization_config::OrganizationAppConfig;
use crate::domain::organization::Organization;
use crate::domain::organization_repository::OrganizationRepository;
use crate::infrastructure::postgres_organization_repository::PostgresOrganizationRepository;
use database::DatabaseConnection;
use shared::entities::AsyncAddEntityUseCase;
use shared::entities::use_case::use_case_traits::AsyncUpdateEntityUseCase;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<OrganizationAppConfig>,
    pub db: DatabaseConnection,
    pub add_organization_use_case: Arc<dyn AsyncAddEntityUseCase<AddOrganizationRequest, Organization>>,
    pub update_organization_use_case: Arc<dyn AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>>,
}

impl AppState {
    pub fn new(config: Arc<OrganizationAppConfig>, db: DatabaseConnection) -> Self {
        let repository = PostgresOrganizationRepository::from_connection(&db);
        let uow = db.unit_of_work();
        Self {
            config,
            db,
            add_organization_use_case: Arc::new(AddOrganizationServiceImpl::new_with_uow(repository.clone(), uow.clone())),
            update_organization_use_case: Arc::new(UpdateOrganizationServiceImpl::new_with_uow(repository, uow)),
        }
    }

    pub fn new_with_repository(
        config: Arc<OrganizationAppConfig>,
        db: DatabaseConnection,
        repository: impl OrganizationRepository + Clone + 'static,
    ) -> Self {
        Self {
            config,
            db,
            add_organization_use_case: Arc::new(AddOrganizationServiceImpl::new(repository.clone())),
            update_organization_use_case: Arc::new(UpdateOrganizationServiceImpl::new(repository)),
        }
    }

    pub fn new_with_use_case(
        config: Arc<OrganizationAppConfig>,
        db: DatabaseConnection,
        add_use_case: Arc<dyn AsyncAddEntityUseCase<AddOrganizationRequest, Organization>>,
        update_use_case: Arc<dyn AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>>,
    ) -> Self {
        Self {
            config,
            db,
            add_organization_use_case: add_use_case,
            update_organization_use_case: update_use_case,
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn config(&self) -> &OrganizationAppConfig {
        &self.config
    }
}
