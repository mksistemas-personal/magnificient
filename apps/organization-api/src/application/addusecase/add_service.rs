use crate::application::OrganizationBusinessErrors;
use crate::application::addusecase::data::{
    AddOrganizationRequest, AddressValidatedContextData, CreatedContextData, InitialContextData,
    ValidatedContextData,
};
use crate::application::addusecase::states::{AddressValidated, Built, Initial, OrganizationValidated};
use crate::domain::{Organization, OrganizationId, OrganizationRepository};
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::use_case::context::Context;
use shared::entities::{AddressRepository, AsyncAddEntityUseCase, AuditFields, BusinessError, Document};
use std::sync::Arc;
use tsid::create_tsid;

trait ValidateInitialContext {
    fn validate(self) -> Result<Context<OrganizationValidated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<OrganizationValidated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        let organization_id = OrganizationId::new(create_tsid());

        let name = request.name.clone();
        if name.trim().is_empty() {
            return Err(OrganizationBusinessErrors::name_empty());
        }

        let document =
            Document::new_validated(&request.document_value, request.document_type.clone())
                .map_err(|_| OrganizationBusinessErrors::document_invalid())?;

        let created_by = request.created_by.clone();
        if created_by.trim().is_empty() {
            return Err(OrganizationBusinessErrors::creator_required());
        }

        let audit = AuditFields::created(&request.created_by);

        Ok(Context::<OrganizationValidated, ValidatedContextData>::new(
            ValidatedContextData {
                request,
                organization_id,
                document,
                audit,
            },
        ))
    }
}

trait ValidateAddressContext {
    fn validate_address(
        self,
    ) -> Result<Context<AddressValidated, AddressValidatedContextData>, BusinessError>;
}

impl ValidateAddressContext for Context<OrganizationValidated, ValidatedContextData> {
    fn validate_address(
        self,
    ) -> Result<Context<AddressValidated, AddressValidatedContextData>, BusinessError> {
        let request = self.data.request;
        let organization_id = self.data.organization_id;
        let document = self.data.document;
        let audit = self.data.audit;

        let address = if let Some(addr) = request.address.clone() {
            addr.validate()?;
            Some(addr)
        } else {
            None
        };

        Ok(Context::<AddressValidated, AddressValidatedContextData>::new(
            AddressValidatedContextData {
                request,
                organization_id,
                document,
                address,
                audit,
            },
        ))
    }
}

trait BuildOrganizationContext {
    fn build(self) -> Result<Context<Built, CreatedContextData>, BusinessError>;
}

impl BuildOrganizationContext for Context<AddressValidated, AddressValidatedContextData> {
    fn build(self) -> Result<Context<Built, CreatedContextData>, BusinessError> {
        let request = self.data.request;
        let mut organization = Organization::new(
            self.data.organization_id,
            request.name.clone(),
            self.data.document,
        )
        .with_audit(self.data.audit);

        if let Some(address) = self.data.address {
            organization = organization.with_address(address);
        }

        Ok(Context::<Built, CreatedContextData>::new(
            CreatedContextData {
                request,
                organization,
            },
        ))
    }
}

trait PersistOrganizationContext {
    async fn save(
        self,
        repository: &dyn OrganizationRepository,
        address_repository: Option<&dyn AddressRepository>,
    ) -> Result<Organization, BusinessError>;
}

impl PersistOrganizationContext for Context<Built, CreatedContextData> {
    async fn save(
        self,
        repository: &dyn OrganizationRepository,
        address_repository: Option<&dyn AddressRepository>,
    ) -> Result<Organization, BusinessError> {
        let mut organization = self.data.organization;
        if let (Some(address_repo), Some(address)) =
            (address_repository, organization.address().cloned())
        {
            let saved_address = address_repo.save(&address).await?;
            organization = organization.with_address(saved_address);
        }
        repository.save(&organization).await
    }
}

#[derive(Clone)]
pub struct AddOrganizationService {
    repository: Arc<dyn OrganizationRepository>,
    address_repository: Option<Arc<dyn AddressRepository>>,
    uow: Option<UnitOfWork>,
}

impl AddOrganizationService {
    pub fn new(repository: impl OrganizationRepository + 'static) -> Self {
        Self {
            repository: Arc::new(repository),
            address_repository: None,
            uow: None,
        }
    }

    pub fn new_with_uow(
        repository: impl OrganizationRepository + 'static,
        uow: UnitOfWork,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            address_repository: None,
            uow: Some(uow),
        }
    }

    pub fn new_with_repositories(
        repository: impl OrganizationRepository + 'static,
        address_repository: impl AddressRepository + 'static,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            address_repository: Some(Arc::new(address_repository)),
            uow: None,
        }
    }

    pub fn new_with_repositories_and_uow(
        repository: impl OrganizationRepository + 'static,
        address_repository: impl AddressRepository + 'static,
        uow: UnitOfWork,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            address_repository: Some(Arc::new(address_repository)),
            uow: Some(uow),
        }
    }

    pub fn new_with_arc(repository: Arc<dyn OrganizationRepository>) -> Self {
        Self {
            repository,
            address_repository: None,
            uow: None,
        }
    }

    pub fn new_with_arcs(
        repository: Arc<dyn OrganizationRepository>,
        address_repository: Option<Arc<dyn AddressRepository>>,
    ) -> Self {
        Self {
            repository,
            address_repository,
            uow: None,
        }
    }

    pub fn new_with_arcs_and_uow(
        repository: Arc<dyn OrganizationRepository>,
        address_repository: Option<Arc<dyn AddressRepository>>,
        uow: UnitOfWork,
    ) -> Self {
        Self {
            repository,
            address_repository,
            uow: Some(uow),
        }
    }

    pub fn new_with_arc_and_uow(
        repository: Arc<dyn OrganizationRepository>,
        uow: UnitOfWork,
    ) -> Self {
        Self {
            repository,
            address_repository: None,
            uow: Some(uow),
        }
    }

    pub fn with_address_repository(
        mut self,
        address_repository: impl AddressRepository + 'static,
    ) -> Self {
        self.address_repository = Some(Arc::new(address_repository));
        self
    }

    pub fn with_address_repository_arc(
        mut self,
        address_repository: Arc<dyn AddressRepository>,
    ) -> Self {
        self.address_repository = Some(address_repository);
        self
    }

    pub async fn execute(
        &self,
        request: AddOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        <Self as AsyncAddEntityUseCase<AddOrganizationRequest, Organization>>::execute(
            self, request,
        )
        .await
    }

    async fn execute_transactional_pipeline(
        initial: Context<Initial, InitialContextData>,
        repo: &dyn OrganizationRepository,
        address_repo: Option<&dyn AddressRepository>,
    ) -> Result<Organization, BusinessError> {
        initial
            .validate()?
            .validate_address()?
            .build()?
            .save(repo, address_repo)
            .await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<AddOrganizationRequest, Organization> for AddOrganizationService {
    async fn execute(
        &self,
        request: AddOrganizationRequest,
    ) -> Result<Organization, BusinessError> {
        let initial = Context::<Initial, _>::new(InitialContextData { request });
        let address_repo = self.address_repository.as_deref();

        if let Some(uow) = &self.uow {
            uow.run_tx(|_tx| {
                Self::execute_transactional_pipeline(
                    initial,
                    self.repository.as_ref(),
                    address_repo,
                )
            })
            .await
        } else {
            Self::execute_transactional_pipeline(
                initial,
                self.repository.as_ref(),
                address_repo,
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{Address, DocumentType};

    #[tokio::test]
    async fn test_add_organization_without_address() {
        let repo = InMemoryOrganizationRepository::new();
        let service = AddOrganizationService::new(repo);

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
        assert!(org.audit.is_some());
        assert_eq!(org.audit.as_ref().unwrap().created_by(), "admin");
    }

    #[tokio::test]
    async fn test_add_organization_with_valid_brazil_address() {
        let repo = InMemoryOrganizationRepository::new();
        let service = AddOrganizationService::new(repo);

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
        .with_address(address.clone());

        let result = service.execute(request).await;
        assert!(result.is_ok());

        let org = result.unwrap();
        assert_eq!(org.name(), "Empresa Teste BR");
        assert_eq!(org.address(), Some(&address));
    }

    #[tokio::test]
    async fn test_add_organization_with_valid_argentina_address() {
        let repo = InMemoryOrganizationRepository::new();
        let service = AddOrganizationService::new(repo);

        let address = Address::new(
            "Av. Corrientes",
            "1234",
            "Piso 4",
            "San Nicolás",
            "Buenos Aires",
            "Buenos Aires",
            "C",
            "C1043AAX",
            "Argentina",
            "AR",
            "",
        );

        let request = AddOrganizationRequest::new(
            "Empresa Teste AR",
            "20-12345678-6",
            DocumentType::Cuit,
            "admin",
        )
        .with_address(address.clone());

        let result = service.execute(request).await;
        assert!(result.is_ok());

        let org = result.unwrap();
        assert_eq!(org.name(), "Empresa Teste AR");
        assert_eq!(org.address(), Some(&address));
    }

    #[tokio::test]
    async fn test_add_organization_with_valid_us_address() {
        use shared::entities::Country;

        let repo = InMemoryOrganizationRepository::new();
        let service = AddOrganizationService::new(repo);

        let address = Address::new(
            "100 Main St",
            "Suite 200",
            "",
            "Downtown",
            "Springfield",
            "Illinois",
            "IL",
            "62701",
            "United States",
            "US",
            "",
        );

        let request = AddOrganizationRequest::new(
            "Empresa Teste US",
            "12-3456789",
            DocumentType::Other {
                name: "EIN".to_string(),
                country: Some(Country::from("US")),
            },
            "admin",
        )
        .with_address(address.clone());

        let result = service.execute(request).await;
        assert!(result.is_ok());

        let org = result.unwrap();
        assert_eq!(org.name(), "Empresa Teste US");
        assert_eq!(org.address(), Some(&address));
    }

    #[tokio::test]
    async fn test_add_organization_with_invalid_brazil_postal_code() {
        let repo = InMemoryOrganizationRepository::new();
        let service = AddOrganizationService::new(repo);

        let address = Address::new(
            "Av. Paulista",
            "1000",
            "",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "123", // CEP inválido (menos de 8 dígitos)
            "Brasil",
            "BR",
            "",
        );

        let request = AddOrganizationRequest::new(
            "Empresa Teste",
            "00.000.000/0001-91",
            DocumentType::Cnpj,
            "admin",
        )
        .with_address(address);

        let result = service.execute(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BusinessError::InvalidData(_)));
    }

    #[tokio::test]
    async fn test_add_organization_with_invalid_brazil_state_code() {
        let repo = InMemoryOrganizationRepository::new();
        let service = AddOrganizationService::new(repo);

        let address = Address::new(
            "Av. Paulista",
            "1000",
            "",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "INVALID_UF",
            "01310-100",
            "Brasil",
            "BR",
            "",
        );

        let request = AddOrganizationRequest::new(
            "Empresa Teste",
            "00.000.000/0001-91",
            DocumentType::Cnpj,
            "admin",
        )
        .with_address(address);

        let result = service.execute(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_organization_with_empty_address_field() {
        let repo = InMemoryOrganizationRepository::new();
        let service = AddOrganizationService::new(repo);

        let address = Address::new(
            "", // Rua vazia
            "1000",
            "",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "01310-100",
            "Brasil",
            "BR",
            "",
        );

        let request = AddOrganizationRequest::new(
            "Empresa Teste",
            "00.000.000/0001-91",
            DocumentType::Cnpj,
            "admin",
        )
        .with_address(address);

        let result = service.execute(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_organization_with_empty_name() {
        let repo = InMemoryOrganizationRepository::new();
        let service = AddOrganizationService::new(repo);

        let request = AddOrganizationRequest::new(
            "  ",
            "00.000.000/0001-91",
            DocumentType::Cnpj,
            "admin",
        );

        let result = service.execute(request).await;
        assert_eq!(result, Err(OrganizationBusinessErrors::name_empty()));
    }

    #[tokio::test]
    async fn test_add_organization_with_invalid_document() {
        let repo = InMemoryOrganizationRepository::new();
        let service = AddOrganizationService::new(repo);

        let request = AddOrganizationRequest::new(
            "Empresa Teste",
            "invalid_doc",
            DocumentType::Cnpj,
            "admin",
        );

        let result = service.execute(request).await;
        assert_eq!(result, Err(OrganizationBusinessErrors::document_invalid()));
    }

    #[tokio::test]
    async fn test_add_organization_with_empty_creator() {
        let repo = InMemoryOrganizationRepository::new();
        let service = AddOrganizationService::new(repo);

        let request = AddOrganizationRequest::new(
            "Empresa Teste",
            "00.000.000/0001-91",
            DocumentType::Cnpj,
            "  ",
        );

        let result = service.execute(request).await;
        assert_eq!(result, Err(OrganizationBusinessErrors::creator_required()));
    }

    #[tokio::test]
    async fn test_add_organization_persists_address_in_address_repository() {
        use shared::entities::InMemoryAddressRepository;

        let org_repo = InMemoryOrganizationRepository::new();
        let addr_repo = InMemoryAddressRepository::new();
        let service = AddOrganizationService::new_with_repositories(org_repo.clone(), addr_repo.clone());

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
            "Empresa Teste Com Repositorio",
            "00.000.000/0001-91",
            DocumentType::Cnpj,
            "admin",
        )
        .with_address(address.clone());

        let result = service.execute(request).await;
        assert!(result.is_ok());

        let org = result.unwrap();
        assert_eq!(org.name(), "Empresa Teste Com Repositorio");
        assert!(org.address().is_some());
        let saved_addr = org.address().unwrap();
        assert!(saved_addr.id().is_some());

        let addr_id = saved_addr.id().unwrap();
        let found_addr = addr_repo.find_by_id(addr_id).await;
        assert!(found_addr.is_ok());
        let found_addr = found_addr.unwrap();
        assert_eq!(found_addr.street, "Av. Paulista");
        assert_eq!(found_addr.city, "São Paulo");
        assert_eq!(found_addr.id(), Some(addr_id));
    }
}
