use crate::application::OrganizationPersistence;
use crate::application::service_constructors::FromPersistence;
use crate::application::update_usecase::data::*;
use crate::application::update_usecase::states::*;
use crate::application::update_usecase::steps::{
    ChangeOrganizationContext, LoadOrganizationContext, PersistOrganizationContext,
    ValidateAddressContext, ValidateInitialContext,
};
use crate::domain::Organization;
use async_trait::async_trait;
use shared::entities::BusinessError;
use shared::entities::use_case::context::Context;
use shared::entities::use_case::use_case_traits::AsyncUpdateEntityUseCase;

#[derive(Clone)]
pub struct UpdateOrganizationService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for UpdateOrganizationService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl UpdateOrganizationService {
    pub async fn execute(
        &self,
        request: UpdateOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        <Self as AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>>::execute(
            self, request,
        )
        .await
    }
}

#[async_trait]
impl AsyncUpdateEntityUseCase<UpdateOrganizationRequest, Organization>
    for UpdateOrganizationService
{
    async fn execute(
        &self,
        request: UpdateOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        // O OrganizationPersistence::run() já encapsula a lógica de transação
        // (com ou sem UnitOfWork), então não precisamos mais duplicar os caminhos
        // com/sem UoW aqui. O load e o save acontecem em chamadas separadas ao run(),
        // cada uma em sua própria transação (ou sem transação se não houver UoW).
        // Para o caso transacional completo load+save, o UoW garante que ambas as
        // chamadas a run() usem a mesma transação.
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .validate_address()?
            .load(&self.persistence)
            .await?
            .change()?
            .save(&self.persistence)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service_constructors::FromPersistence;
    use crate::domain::{Organization, OrganizationId, OrganizationRepository};
    use crate::errors::OrganizationBusinessErrors;
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::address::repository::InMemoryAddressRepository;
    use shared::entities::{Address, AddressRepository, AuditFields, Document, DocumentType, TSID};

    async fn seed(repo: &InMemoryOrganizationRepository, id: u64) -> Organization {
        let org_id = OrganizationId::new(TSID::from(id));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Nome Original", doc)
            .with_audit(AuditFields::created("admin"));
        let conn: &mut sqlx::PgConnection = unsafe { &mut *std::ptr::NonNull::dangling().as_ptr() };
        OrganizationRepository::save(repo, conn, &org)
            .await
            .unwrap();
        org
    }

    #[tokio::test]
    async fn test_update_organization_service_full_flow() {
        let repo = InMemoryOrganizationRepository::new();
        seed(&repo, 100).await;

        let service = UpdateOrganizationService::new_for_tests(repo);

        let update_req = UpdateOrganizationRequest {
            organization_id: OrganizationId::new(TSID::from(100u64)),
            name: "Novo Nome".to_string(),
            address: None,
            updated_by: "editor".to_string(),
        };

        let result = service.execute(update_req).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name(), "Novo Nome");
        let audit = updated.audit().unwrap();
        assert_eq!(audit.created_by(), "admin");
        assert_eq!(audit.updated_by(), "editor");
    }

    #[tokio::test]
    async fn test_update_organization_service_with_address() {
        let repo = InMemoryOrganizationRepository::new();
        let address_repo = InMemoryAddressRepository::new();
        seed(&repo, 100).await;

        let service =
            UpdateOrganizationService::new_with_repositories_for_tests(repo, address_repo.clone());

        let address = Address::new(
            "Av. Paulista",
            "1000",
            "Conjunto 101",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "01310-100",
            "Brasil",
            "BR",
            "Próximo ao metrô",
        );

        let update_req = UpdateOrganizationRequest::new(
            OrganizationId::new(TSID::from(100u64)),
            "Novo Nome",
            "editor",
        )
        .with_address(address.clone());

        let result = service.execute(update_req).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name(), "Novo Nome");
        assert!(updated.address().is_some());
        let saved_addr = updated.address().unwrap();
        assert_eq!(saved_addr.street, "Av. Paulista");
        assert!(saved_addr.id().is_some());

        let addr_in_repo = address_repo.find_by_id(saved_addr.id().unwrap()).await;
        assert!(addr_in_repo.is_ok());
    }

    #[tokio::test]
    async fn test_update_organization_service_with_invalid_address() {
        let repo = InMemoryOrganizationRepository::new();
        let address_repo = InMemoryAddressRepository::new();
        seed(&repo, 100).await;

        let service =
            UpdateOrganizationService::new_with_repositories_for_tests(repo, address_repo);

        let invalid_address = Address::new(
            "Av. Paulista",
            "1000",
            "",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "123", // CEP inválido para Brasil
            "Brasil",
            "BR",
            "",
        );

        let update_req = UpdateOrganizationRequest::new(
            OrganizationId::new(TSID::from(100u64)),
            "Novo Nome",
            "editor",
        )
        .with_address(invalid_address);

        let result = service.execute(update_req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_organization_service_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let service = UpdateOrganizationService::new_for_tests(repo);

        let update_req = UpdateOrganizationRequest {
            organization_id: OrganizationId::new(TSID::from(999u64)),
            name: "Empresa Inexistente".to_string(),
            address: None,
            updated_by: "editor".to_string(),
        };

        let result = service.execute(update_req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_update_organization_validation_errors() {
        let repo = InMemoryOrganizationRepository::new();
        let service = UpdateOrganizationService::new_for_tests(repo);
        let org_id = OrganizationId::new(TSID::from(100u64));

        let invalid_name_req = UpdateOrganizationRequest {
            organization_id: org_id,
            name: "".to_string(),
            address: None,
            updated_by: "editor".to_string(),
        };
        assert_eq!(
            service.execute(invalid_name_req).await.unwrap_err(),
            OrganizationBusinessErrors::name_empty()
        );

        let invalid_updater_req = UpdateOrganizationRequest {
            organization_id: org_id,
            name: "Nome".to_string(),
            address: None,
            updated_by: "".to_string(),
        };
        assert_eq!(
            service.execute(invalid_updater_req).await.unwrap_err(),
            OrganizationBusinessErrors::updater_required()
        );
    }
}
