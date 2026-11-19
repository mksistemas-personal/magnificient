use crate::domain::{Supplier, SupplierRepository};
use crate::errors::SupplierBusinessErrors;
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
pub struct InMemorySupplierRepository {
    items: Arc<RwLock<HashMap<TSID, Supplier>>>,
}

impl InMemorySupplierRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SupplierRepository for InMemorySupplierRepository {
    async fn save(
        &self,
        _conn: &mut PgConnection,
        supplier: &Supplier,
    ) -> Result<Supplier, BusinessError> {
        self.items
            .write()
            .expect("supplier repository lock poisoned")
            .insert(supplier.id().value(), supplier.clone());
        Ok(supplier.clone())
    }

    async fn find_by_id(
        &self,
        _conn: &mut PgConnection,
        id: TSID,
    ) -> Result<Supplier, BusinessError> {
        self.items
            .read()
            .expect("supplier repository lock poisoned")
            .get(&id)
            .cloned()
            .ok_or_else(|| SupplierBusinessErrors::not_found(id))
    }

    async fn delete(
        &self,
        _conn: &mut PgConnection,
        id: TSID,
        _updated_by: &str,
    ) -> Result<(), BusinessError> {
        self.items
            .write()
            .expect("supplier repository lock poisoned")
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| SupplierBusinessErrors::not_found(id))
    }

    async fn find_page(
        &self,
        _conn: &mut PgConnection,
        name: Option<&str>,
        organization_id: Option<TSID>,
        page_request: &PageRequest,
    ) -> Result<Page<Supplier>, BusinessError> {
        let name = name.map(str::to_lowercase);
        let mut suppliers: Vec<_> = self
            .items
            .read()
            .expect("supplier repository lock poisoned")
            .values()
            .filter(|supplier| {
                name.as_ref()
                    .is_none_or(|name| supplier.name().to_lowercase().contains(name))
                    && organization_id
                        .is_none_or(|id| supplier.organization_id().number() == id.number())
            })
            .cloned()
            .collect();
        suppliers.sort_by(|left, right| left.name().cmp(right.name()).then(left.id().cmp(&right.id())));

        let total = suppliers.len() as u64;
        let start = page_request.offset() as usize;
        let content = suppliers
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
