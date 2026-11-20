use crate::domain::{Product, ProductRepository};
use crate::errors::ProductBusinessErrors;
use async_trait::async_trait;
use shared::entities::BusinessError;
use shared::types::{Page, PageRequest};
use sqlx::PgConnection;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tsid::TSID;

#[derive(Debug, Clone, Default)]
pub struct InMemoryProductRepository {
    items: Arc<RwLock<HashMap<TSID, Product>>>,
}

impl InMemoryProductRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ProductRepository for InMemoryProductRepository {
    async fn save(
        &self,
        _conn: &mut PgConnection,
        product: &Product,
    ) -> Result<Product, BusinessError> {
        self.items
            .write()
            .expect("product repository lock poisoned")
            .insert(product.id().value(), product.clone());
        Ok(product.clone())
    }

    async fn find_by_id(
        &self,
        _conn: &mut PgConnection,
        id: TSID,
    ) -> Result<Product, BusinessError> {
        self.items
            .read()
            .expect("product repository lock poisoned")
            .get(&id)
            .cloned()
            .ok_or_else(|| ProductBusinessErrors::not_found(id))
    }

    async fn delete(
        &self,
        _conn: &mut PgConnection,
        id: TSID,
        _updated_by: &str,
    ) -> Result<(), BusinessError> {
        self.items
            .write()
            .expect("product repository lock poisoned")
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| ProductBusinessErrors::not_found(id))
    }

    async fn find_page(
        &self,
        _conn: &mut PgConnection,
        name: Option<&str>,
        organization_id: Option<TSID>,
        page_request: &PageRequest,
    ) -> Result<Page<Product>, BusinessError> {
        let name = name.map(str::to_lowercase);
        let mut products: Vec<_> = self
            .items
            .read()
            .expect("product repository lock poisoned")
            .values()
            .filter(|product| {
                name.as_ref()
                    .is_none_or(|name| product.name().to_lowercase().contains(name))
                    && organization_id
                        .is_none_or(|id| product.organization_id().number() == id.number())
            })
            .cloned()
            .collect();
        products.sort_by(|left, right| left.name().cmp(right.name()).then(left.id().cmp(&right.id())));

        let total = products.len() as u64;
        let start = page_request.offset() as usize;
        let content = products
            .into_iter()
            .skip(start)
            .take(page_request.limit() as usize)
            .collect();
        Ok(Page::new(
            content,
            page_request.page,
            page_request.page_size,
            total,
        ))
    }
}
