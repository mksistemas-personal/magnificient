use crate::domain::{Product, ProductRepository};
use database::{ConnectionExecutor, UnitOfWork};
use shared::entities::{BusinessError};
use shared::types::{Page, PageRequest};
use sqlx::PgPool;
use std::sync::Arc;
use tsid::TSID;

#[derive(Clone)]
pub struct ProductPersistence {
    repository: Arc<dyn ProductRepository>,
    executor: ConnectionExecutor,
}

impl ProductPersistence {
    pub fn new(
        repository: impl ProductRepository + 'static,
        pool: PgPool,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            executor: ConnectionExecutor::new(pool),
        }
    }

    pub fn new_without_pool(
        repository: impl ProductRepository + 'static,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            executor: ConnectionExecutor::new_without_pool(),
        }
    }

    pub fn with_unit_of_work(mut self, unit_of_work: UnitOfWork) -> Self {
        self.executor = self.executor.with_unit_of_work(unit_of_work);
        self
    }

    pub async fn save(&self, product: Product) -> Result<Product, BusinessError> {
        let repository = Arc::clone(&self.repository);
        self.executor
            .run(move |conn| {
                Box::pin(async move {
                    repository.save(conn, &product).await
                })
            })
            .await
    }

    pub async fn find_by_id(&self, id: TSID) -> Result<Product, BusinessError> {
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
    ) -> Result<Page<Product>, BusinessError> {
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
