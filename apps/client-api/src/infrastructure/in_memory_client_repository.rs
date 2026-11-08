use crate::domain::{Client, ClientRepository};
use crate::errors::ClientBusinessErrors;
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
pub struct InMemoryClientRepository {
    items: Arc<RwLock<HashMap<TSID, Client>>>,
}

impl InMemoryClientRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ClientRepository for InMemoryClientRepository {
    async fn save(
        &self,
        _conn: &mut PgConnection,
        client: &Client,
    ) -> Result<Client, BusinessError> {
        self.items
            .write()
            .expect("client repository lock poisoned")
            .insert(client.id.value(), client.clone());
        Ok(client.clone())
    }

    async fn find_by_id(
        &self,
        _conn: &mut PgConnection,
        id: TSID,
    ) -> Result<Client, BusinessError> {
        self.items
            .read()
            .expect("client repository lock poisoned")
            .get(&id)
            .cloned()
            .ok_or_else(|| ClientBusinessErrors::not_found(id))
    }

    async fn delete(
        &self,
        _conn: &mut PgConnection,
        id: TSID,
        _updated_by: &str,
    ) -> Result<(), BusinessError> {
        self.items
            .write()
            .expect("client repository lock poisoned")
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| ClientBusinessErrors::not_found(id))
    }

    async fn find_page(
        &self,
        _conn: &mut PgConnection,
        name: Option<&str>,
        organization_id: Option<TSID>,
        page_request: &PageRequest,
    ) -> Result<Page<Client>, BusinessError> {
        let name = name.map(str::to_lowercase);
        let mut clients: Vec<_> = self
            .items
            .read()
            .expect("client repository lock poisoned")
            .values()
            .filter(|client| {
                name.as_ref()
                    .is_none_or(|name| client.name.to_lowercase().contains(name))
                    && organization_id
                        .is_none_or(|id| client.organization_id.number() == id.number())
            })
            .cloned()
            .collect();
        clients.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));

        let total = clients.len() as u64;
        let start = page_request.offset() as usize;
        let content = clients
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
