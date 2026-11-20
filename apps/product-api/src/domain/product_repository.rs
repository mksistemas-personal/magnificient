use crate::domain::Product;
use async_trait::async_trait;
use shared::entities::BusinessError;
use shared::types::{PageRequest, page::Page};
use sqlx::PgConnection;
use tsid::TSID;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn save(&self, conn: &mut PgConnection, product: &Product)
    -> Result<Product, BusinessError>;

    async fn find_by_id(&self, conn: &mut PgConnection, id: TSID) -> Result<Product, BusinessError>;

    async fn delete(
        &self,
        conn: &mut PgConnection,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError>;

    async fn find_page(
        &self,
        conn: &mut PgConnection,
        name: Option<&str>,
        organization_id: Option<TSID>,
        page_request: &PageRequest,
    ) -> Result<Page<Product>, BusinessError>;
}
