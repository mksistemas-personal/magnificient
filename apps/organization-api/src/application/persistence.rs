use crate::domain::{Organization, OrganizationLink, OrganizationRepository};
use database::{ConnectionExecutor, UnitOfWork};
use shared::entities::{AddressRepository, BusinessError};
use shared::types::PageRequest;
use shared::types::page::Page;
use sqlx::PgPool;
use std::sync::Arc;
use tsid::TSID;

/// Persistence dependencies and transaction policy shared by organization use cases.
#[derive(Clone)]
pub struct OrganizationPersistence {
    pub(crate) repository: Arc<dyn OrganizationRepository>,
    pub(crate) address_repository: Option<Arc<dyn AddressRepository>>,
    executor: ConnectionExecutor,
}

impl OrganizationPersistence {
    pub fn new(repository: impl OrganizationRepository + 'static, pool: PgPool) -> Self {
        Self {
            repository: Arc::new(repository),
            address_repository: None,
            executor: ConnectionExecutor::new(pool),
        }
    }

    /// Cria uma instância sem pool de banco de dados, adequada apenas para uso com
    /// repositórios in-memory (testes unitários). Não pode ser usado com UnitOfWork.
    pub fn new_without_pool(repository: impl OrganizationRepository + 'static) -> Self {
        Self {
            repository: Arc::new(repository),
            address_repository: None,
            executor: ConnectionExecutor::new_without_pool(),
        }
    }

    pub fn with_address_repository(
        mut self,
        address_repository: impl AddressRepository + 'static,
    ) -> Self {
        self.address_repository = Some(Arc::new(address_repository));
        self
    }

    pub fn with_unit_of_work(mut self, unit_of_work: UnitOfWork) -> Self {
        self.executor = self.executor.with_unit_of_work(unit_of_work);
        self
    }

    pub async fn save(&self, organization: Organization) -> Result<Organization, BusinessError> {
        let repository = Arc::clone(&self.repository);
        let address_repository = self.address_repository.clone();
        self.executor
            .run(move |conn| {
                Box::pin(async move {
                    let mut org = organization;
                    if let (Some(addr_repo), Some(address)) =
                        (&address_repository, org.address().cloned())
                    {
                        let saved_addr = addr_repo.save_in_tx(conn, &address).await?;
                        org = org.with_address(saved_addr);
                    }
                    repository.save(conn, &org).await
                })
            })
            .await
    }

    pub async fn find_by_id(&self, id: TSID) -> Result<Organization, BusinessError> {
        let repository = Arc::clone(&self.repository);
        self.executor
            .run(move |conn| Box::pin(async move { repository.find_by_id(conn, id).await }))
            .await
    }

    pub async fn delete(&self, id: TSID, updated_by: &str) -> Result<(), BusinessError> {
        let repository = Arc::clone(&self.repository);
        let updated_by = updated_by.to_owned();
        self.executor
            .run(move |conn| {
                Box::pin(async move { repository.delete(conn, id, &updated_by).await })
            })
            .await
    }

    pub async fn find_page(
        &self,
        name: Option<&str>,
        page_request: PageRequest,
    ) -> Result<Page<Organization>, BusinessError> {
        let repository = Arc::clone(&self.repository);
        let name = name.map(str::to_owned);
        self.executor
            .run(move |conn| {
                Box::pin(async move {
                    let content = repository
                        .find_all(conn, name.as_deref(), &page_request)
                        .await?;
                    let total = repository.count(conn, name.as_deref()).await?;
                    Ok(Page::new(
                        content,
                        page_request.page,
                        page_request.page_size,
                        total,
                    ))
                })
            })
            .await
    }

    pub async fn link_entity(
        &self,
        link: OrganizationLink,
    ) -> Result<OrganizationLink, BusinessError> {
        let repository = Arc::clone(&self.repository);
        self.executor
            .run(move |conn| {
                Box::pin(async move {
                    repository.link_entity(conn, &link).await
                })
            })
            .await
    }
}
