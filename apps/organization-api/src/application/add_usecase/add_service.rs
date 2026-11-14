use crate::application::OrganizationPersistence;
use crate::application::add_usecase::data::{AddOrganizationRequest, InitialContextData};
use crate::application::add_usecase::states::Initial;
use crate::application::add_usecase::steps::{
    BuildOrganizationContext, PersistOrganizationContext, ValidateAddressContext,
    ValidateInitialContext,
};
use crate::application::service_constructors::FromPersistence;
use crate::domain::Organization;
use async_trait::async_trait;
use shared::entities::{AsyncAddEntityUseCase, BusinessError, Context};

#[derive(Clone)]
pub struct AddOrganizationService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for AddOrganizationService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl AddOrganizationService {
    pub async fn execute(
        &self,
        request: AddOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        <Self as AsyncAddEntityUseCase<AddOrganizationRequest, Organization>>::execute(
            self, request,
        )
        .await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<AddOrganizationRequest, Organization> for AddOrganizationService {
    async fn execute(
        &self,
        request: AddOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .validate_address()?
            .build()?
            .save(&self.persistence)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service_constructors::FromPersistence;
    use crate::errors::OrganizationBusinessErrors;
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{Address, DocumentType};

    fn make_service() -> AddOrganizationService {
        let repo = InMemoryOrganizationRepository::new();
        AddOrganizationService::new_for_tests(repo)
    }

    #[tokio::test]
    async fn test_add_organization_without_address() {
        let service = make_service();

        let request = AddOrganizationRequest::new(
            "Empresa Teste",
            "00.000.000/0001-91",
            DocumentType::Cnpj,
            "admin",
        );

        let result = service.execute(request).await;
        assert!(result.is_ok());

        let org = result.unwrap();
        assert_eq!(org.name(), "Empresa Teste");
        assert_eq!(org.document().value(), "00.000.000/0001-91");
        assert_eq!(org.address(), None);
        assert!(org.audit().is_some());
        assert_eq!(org.audit().unwrap().created_by(), "admin");
    }

    #[tokio::test]
    async fn test_add_organization_with_valid_brazil_address() {
        let repo = InMemoryOrganizationRepository::new();
        let address_repo = shared::entities::InMemoryAddressRepository::new();
        let service = AddOrganizationService::new_with_repositories_for_tests(repo, address_repo);

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

        let request = AddOrganizationRequest::new(
            "Empresa Teste BR",
            "00.000.000/0001-91",
            DocumentType::Cnpj,
            "admin",
        )
        .with_address(address);

        let result = service.execute(request).await;
        assert!(result.is_ok());
        let org = result.unwrap();
        assert!(org.address().is_some());
    }

    #[tokio::test]
    async fn test_add_organization_with_empty_name() {
        let service = make_service();

        let request =
            AddOrganizationRequest::new("", "00.000.000/0001-91", DocumentType::Cnpj, "admin");
        let result = service.execute(request).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            OrganizationBusinessErrors::name_empty()
        );
    }

    #[tokio::test]
    async fn test_add_organization_with_empty_creator() {
        let service = make_service();

        let request = AddOrganizationRequest::new(
            "Empresa Teste",
            "00.000.000/0001-91",
            DocumentType::Cnpj,
            "",
        );
        let result = service.execute(request).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            OrganizationBusinessErrors::creator_required()
        );
    }
}
