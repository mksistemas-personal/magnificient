use crate::domain::{Client, ClientRepository};
use database::{ConnectionExecutor, UnitOfWork};
use shared::entities::{AddressRepository, BusinessError};
use shared::types::{Page, PageRequest};
use sqlx::PgPool;
use std::sync::Arc;
use tsid::TSID;

#[derive(Clone)]
pub struct ClientPersistence {
    repository: Arc<dyn ClientRepository>,
    address_repository: Arc<dyn AddressRepository>,
    executor: ConnectionExecutor,
}

impl ClientPersistence {
    pub fn new(
        repository: impl ClientRepository + 'static,
        address_repository: impl AddressRepository + 'static,
        pool: PgPool,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            address_repository: Arc::new(address_repository),
            executor: ConnectionExecutor::new(pool),
        }
    }

    pub fn new_without_pool(
        repository: impl ClientRepository + 'static,
        address_repository: impl AddressRepository + 'static,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            address_repository: Arc::new(address_repository),
            executor: ConnectionExecutor::new_without_pool(),
        }
    }

    pub fn with_unit_of_work(mut self, unit_of_work: UnitOfWork) -> Self {
        self.executor = self.executor.with_unit_of_work(unit_of_work);
        self
    }

    pub async fn save(&self, client: Client) -> Result<Client, BusinessError> {
        let repository = Arc::clone(&self.repository);
        let address_repository = Arc::clone(&self.address_repository);
        self.executor
            .run(move |conn| {
                Box::pin(async move {
                    let mut client = client;
                    if let Some(address) = client.second_address.clone() {
                        client.second_address =
                            Some(address_repository.save_in_tx(conn, &address).await?);
                    }
                    repository.save(conn, &client).await
                })
            })
            .await
    }

    pub async fn find_by_id(&self, id: TSID) -> Result<Client, BusinessError> {
        let repository = Arc::clone(&self.repository);
        self.executor
            .run(move |conn| Box::pin(async move { repository.find_by_id(conn, id).await }))
            .await
    }

    pub async fn delete(&self, id: TSID, updated_by: String) -> Result<(), BusinessError> {
        let repository = Arc::clone(&self.repository);
        self.executor
            .run(move |conn| {
                Box::pin(async move { repository.delete(conn, id, &updated_by).await })
            })
            .await
    }

    pub async fn find_page(
        &self,
        name: Option<String>,
        organization_id: Option<TSID>,
        page_request: PageRequest,
    ) -> Result<Page<Client>, BusinessError> {
        let repository = Arc::clone(&self.repository);
        self.executor
            .run(move |conn| {
                Box::pin(async move {
                    repository
                        .find_page(conn, name.as_deref(), organization_id, &page_request)
                        .await
                })
            })
            .await
    }
}
