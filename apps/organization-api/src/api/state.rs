use crate::application::addusecase::add_service::AddOrganizationService;
use crate::application::addusecase::data::AddOrganizationRequest;
use crate::application::allusecase::all_service::FindAllOrganizationsService;
use crate::application::allusecase::data::FindAllOrganizationsRequest;
use crate::application::updateusecase::UpdateOrganizationService;
use crate::application::updateusecase::data::UpdateOrganizationRequest;
use crate::application::{
    DeleteOrganizationRequest, DeleteOrganizationService, FindOrganizationByIdRequest,
    GetOrganizationByIdRequest, GetOrganizationByIdService,
};
use crate::config::organization_config::OrganizationAppConfig;
use crate::domain::organization::Organization;
use crate::domain::organization_repository::OrganizationRepository;
use crate::infrastructure::postgres_organization_repository::PostgresOrganizationRepository;
use database::DatabaseConnection;
use shared::entities::AsyncAddEntityUseCase;
use shared::entities::use_case::use_case_traits::AsyncUpdateEntityUseCase;
use shared::entities::use_case::{AsyncDeleteEntityUseCase, AsyncGetAllUseCase, AsyncGetByIdUseCase};
use shared::types::page::Page;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<OrganizationAppConfig>,
    pub db: DatabaseConnection,
    pub add_organization_use_case:
        Arc<dyn AsyncAddEntityUseCase<AddOrganizationRequest, Organization>>,
    pub update_organization_use_case:
        Arc<dyn AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>>,
    pub delete_organization_use_case:
        Arc<dyn AsyncDeleteEntityUseCase<DeleteOrganizationRequest, ()>>,
    pub get_by_id_organization_use_case:
        Arc<dyn AsyncGetByIdUseCase<FindOrganizationByIdRequest, Organization>>,
    pub get_all_organization_use_case:
        Arc<dyn AsyncGetAllUseCase<FindAllOrganizationsRequest, Page<Organization>>>,
}

impl AppState {
    pub fn new(config: Arc<OrganizationAppConfig>, db: DatabaseConnection) -> Self {
        let repository = PostgresOrganizationRepository::from_connection(&db);
        let uow = db.unit_of_work();
        Self {
            config,
            db,
            add_organization_use_case: Arc::new(AddOrganizationService::new_with_uow(
                repository.clone(),
                uow.clone(),
            )),
            update_organization_use_case: Arc::new(UpdateOrganizationService::new_with_uow(
                repository.clone(),
                uow.clone(),
            )),
            delete_organization_use_case: Arc::new(DeleteOrganizationService::new_with_uow(
                repository.clone(),
                uow.clone(),
            )),
            get_by_id_organization_use_case: Arc::new(GetOrganizationByIdService::new_with_uow(
                repository.clone(),
                uow.clone(),
            )),
            get_all_organization_use_case: Arc::new(FindAllOrganizationsService::new_with_uow(
                repository, uow,
            )),
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
            add_organization_use_case: Arc::new(AddOrganizationService::new(repository.clone())),
            update_organization_use_case: Arc::new(UpdateOrganizationService::new(
                repository.clone(),
            )),
            delete_organization_use_case: Arc::new(DeleteOrganizationService::new(
                repository.clone(),
            )),
            get_by_id_organization_use_case: Arc::new(GetOrganizationByIdService::new(
                repository.clone(),
            )),
            get_all_organization_use_case: Arc::new(FindAllOrganizationsService::new(repository)),
        }
    }

    pub fn new_with_use_case(
        config: Arc<OrganizationAppConfig>,
        db: DatabaseConnection,
        add_use_case: Arc<dyn AsyncAddEntityUseCase<AddOrganizationRequest, Organization>>,
        update_use_case: Arc<dyn AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>>,
        delete_use_case: Arc<dyn AsyncDeleteEntityUseCase<DeleteOrganizationRequest, ()>>,
        get_by_id_use_case: Arc<dyn AsyncGetByIdUseCase<GetOrganizationByIdRequest, Organization>>,
        get_all_use_case: Arc<dyn AsyncGetAllUseCase<FindAllOrganizationsRequest, Page<Organization>>>,
    ) -> Self {
        Self {
            config,
            db,
            add_organization_use_case: add_use_case,
            update_organization_use_case: update_use_case,
            delete_organization_use_case: delete_use_case,
            get_by_id_organization_use_case: get_by_id_use_case,
            get_all_organization_use_case: get_all_use_case,
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn config(&self) -> &OrganizationAppConfig {
        &self.config
    }
}
