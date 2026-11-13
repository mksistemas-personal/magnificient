use crate::domain::OrganizationEntityType;
use crate::domain::organization::{Organization, OrganizationId};
use crate::domain::organization_link::OrganizationLink;
use crate::domain::organization_repository::OrganizationRepository;
use crate::errors::OrganizationBusinessErrors;
use async_trait::async_trait;
use shared::entities::BusinessError;
use shared::types::page::PageRequest;
use sqlx::PgConnection;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tsid::TSID;

/// In-memory implementation of the `Organization` repository, suitable for unit tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryOrganizationRepository {
    items: Arc<RwLock<HashMap<OrganizationId, Organization>>>,
    links: Arc<RwLock<Vec<OrganizationLink>>>,
}

impl InMemoryOrganizationRepository {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
            links: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn len(&self) -> usize {
        self.items.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.read().unwrap().is_empty()
    }

    pub fn find_by_id(&self, id: &OrganizationId) -> Option<Organization> {
        self.items.read().unwrap().get(id).cloned()
    }

    pub fn insert_for_test(&self, organization: Organization) {
        self.items.write().unwrap().insert(organization.id(), organization);
    }

    pub fn insert_link_for_test(&self, link: OrganizationLink) {
        self.links.write().unwrap().push(link);
    }

    pub fn links(&self) -> Vec<OrganizationLink> {
        self.links.read().unwrap().clone()
    }
}

#[async_trait]
impl OrganizationRepository for InMemoryOrganizationRepository {
    async fn save(
        &self,
        _conn: &mut PgConnection,
        organization: &Organization,
    ) -> Result<Organization, BusinessError> {
        let mut map = self.items.write().unwrap();
        // Verifica unicidade de documento
        for existing in map.values() {
            if existing.id() != organization.id()
                && existing.document().value() == organization.document().value()
                && existing.document().document_type() == organization.document().document_type()
            {
                return Err(OrganizationBusinessErrors::document_already_in_use());
            }
        }

        map.insert(organization.id(), organization.clone());
        Ok(organization.clone())
    }

    async fn find_by_id(
        &self,
        _conn: &mut PgConnection,
        id: TSID,
    ) -> Result<Organization, BusinessError> {
        let org_id = OrganizationId::new(id);
        self.items
            .read()
            .unwrap()
            .get(&org_id)
            .cloned()
            .ok_or_else(OrganizationBusinessErrors::not_found)
    }

    async fn delete(
        &self,
        _conn: &mut PgConnection,
        id: TSID,
        _updated_by: &str,
    ) -> Result<(), BusinessError> {
        let org_id = OrganizationId::new(id);
        let mut map = self.items.write().unwrap();
        if map.remove(&org_id).is_none() {
            return Err(OrganizationBusinessErrors::not_found());
        }
        Ok(())
    }

    async fn find_all(
        &self,
        _conn: &mut PgConnection,
        name: Option<&str>,
        page_request: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError> {
        let map = self.items.read().unwrap();
        let mut list: Vec<Organization> = map
            .values()
            .filter(|org| {
                if let Some(filter) = name {
                    let filter = filter.trim();
                    if !filter.is_empty() {
                        return org.name().to_lowercase().contains(&filter.to_lowercase());
                    }
                }
                true
            })
            .cloned()
            .collect();

        list.sort_by_key(|a| a.id());

        let offset = page_request.offset() as usize;
        let limit = page_request.limit() as usize;

        let result = list.into_iter().skip(offset).take(limit).collect();
        Ok(result)
    }

    async fn count(
        &self,
        _conn: &mut PgConnection,
        name: Option<&str>,
    ) -> Result<u64, BusinessError> {
        let map = self.items.read().unwrap();
        let total = map
            .values()
            .filter(|org| {
                if let Some(filter) = name {
                    let filter = filter.trim();
                    if !filter.is_empty() {
                        return org.name().to_lowercase().contains(&filter.to_lowercase());
                    }
                }
                true
            })
            .count() as u64;
        Ok(total)
    }

    async fn link_entity(
        &self,
        _conn: &mut PgConnection,
        link: &OrganizationLink,
    ) -> Result<OrganizationLink, BusinessError> {
        let map = self.items.read().unwrap();
        if !map.contains_key(&link.organization_id()) {
            return Err(OrganizationBusinessErrors::not_found());
        }
        drop(map);

        let mut links = self.links.write().unwrap();
        for existing in links.iter() {
            if existing.organization_id() == link.organization_id()
                && existing.entity_type() == link.entity_type()
                && existing.entity_id() == link.entity_id()
            {
                return Err(OrganizationBusinessErrors::already_exists());
            }
        }
        links.push(link.clone());
        Ok(link.clone())
    }

    async fn unlink_entity(
        &self,
        _conn: &mut PgConnection,
        entity_id: TSID,
        entity_type: OrganizationEntityType,
    ) -> Result<(), BusinessError> {
        let mut links = self.links.write().unwrap();
        links.retain(|l| !(l.entity_id() == entity_id && l.entity_type() == entity_type));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::entities::{AuditFields, Document, DocumentType};

    macro_rules! with_dummy_conn {
        ($body:expr) => {{
            // O InMemoryOrganizationRepository ignora completamente o &mut PgConnection,
            // mas o trait exige o parâmetro. Usamos unsafe somente nos testes para
            // criar uma referência que nunca é desreferenciada.
            let conn: &mut PgConnection = unsafe { &mut *std::ptr::NonNull::dangling().as_ptr() };
            $body(conn)
        }};
    }

    #[tokio::test]
    async fn test_in_memory_find_by_id_success() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Teste", doc)
            .with_audit(AuditFields::created("admin"));

        with_dummy_conn!(|conn| repo.save(conn, &org))
            .await
            .unwrap();

        let found = with_dummy_conn!(|conn| {
            OrganizationRepository::find_by_id(&repo, conn, TSID::from(100u64))
        })
        .await
        .unwrap();
        assert_eq!(found.id(), org_id);
        assert_eq!(found.name(), "Empresa Teste");
    }

    #[tokio::test]
    async fn test_in_memory_find_by_id_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let result = with_dummy_conn!(|conn| {
            OrganizationRepository::find_by_id(&repo, conn, TSID::from(999u64))
        })
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_in_memory_delete_success() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Teste", doc)
            .with_audit(AuditFields::created("admin"));

        with_dummy_conn!(|conn| repo.save(conn, &org))
            .await
            .unwrap();

        let delete_res = with_dummy_conn!(|conn| {
            OrganizationRepository::delete(&repo, conn, TSID::from(100u64), "admin_delete")
        })
        .await;
        assert!(delete_res.is_ok());

        let find_res = with_dummy_conn!(|conn| {
            OrganizationRepository::find_by_id(&repo, conn, TSID::from(100u64))
        })
        .await;
        assert!(find_res.is_err());
        assert_eq!(
            find_res.unwrap_err(),
            OrganizationBusinessErrors::not_found()
        );
    }

    #[tokio::test]
    async fn test_in_memory_delete_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let result = with_dummy_conn!(|conn| {
            OrganizationRepository::delete(&repo, conn, TSID::from(999u64), "admin_delete")
        })
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_in_memory_find_all_with_filter_and_pagination() {
        let repo = InMemoryOrganizationRepository::new();
        for i in 1..=10 {
            let org_id = OrganizationId::new(TSID::from(i as u64));
            let name = if i % 2 == 0 {
                format!("Empresa Alfa {}", i)
            } else {
                format!("Empresa Beta {}", i)
            };
            let doc = Document::new(format!("{:014}", i), DocumentType::Cnpj);
            let org =
                Organization::new(org_id, name, doc).with_audit(AuditFields::created("admin"));
            with_dummy_conn!(|conn| repo.save(conn, &org))
                .await
                .unwrap();
        }

        // Test filter by name "Alfa"
        let page_req = PageRequest::new(1, 10);
        let result = with_dummy_conn!(|conn| {
            OrganizationRepository::find_all(&repo, conn, Some("Alfa"), &page_req)
        })
        .await
        .unwrap();
        assert_eq!(result.len(), 5);
        for org in &result {
            assert!(org.name().contains("Alfa"));
        }

        // Test pagination: page 1, size 3
        let page1 = PageRequest::new(1, 3);
        let result_p1 =
            with_dummy_conn!(|conn| OrganizationRepository::find_all(&repo, conn, None, &page1))
                .await
                .unwrap();
        assert_eq!(result_p1.len(), 3);

        // Test pagination: page 2, size 3
        let page2 = PageRequest::new(2, 3);
        let result_p2 =
            with_dummy_conn!(|conn| OrganizationRepository::find_all(&repo, conn, None, &page2))
                .await
                .unwrap();
        assert_eq!(result_p2.len(), 3);
        assert_ne!(result_p1[0].id(), result_p2[0].id());

        // Test pagination: page 4, size 3 (should have 1 item out of 10)
        let page4 = PageRequest::new(4, 3);
        let result_p4 =
            with_dummy_conn!(|conn| OrganizationRepository::find_all(&repo, conn, None, &page4))
                .await
                .unwrap();
        assert_eq!(result_p4.len(), 1);

        // Test count
        let total_all = with_dummy_conn!(|conn| OrganizationRepository::count(&repo, conn, None))
            .await
            .unwrap();
        assert_eq!(total_all, 10);

        let total_alfa =
            with_dummy_conn!(|conn| OrganizationRepository::count(&repo, conn, Some("Alfa")))
                .await
                .unwrap();
        assert_eq!(total_alfa, 5);
    }

    #[tokio::test]
    async fn test_in_memory_link_entity_success() {
        use crate::domain::{OrganizationEntityType, OrganizationLink};

        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Teste", doc)
            .with_audit(AuditFields::created("admin"));

        with_dummy_conn!(|conn| repo.save(conn, &org))
            .await
            .unwrap();

        let link = OrganizationLink::new(
            org_id,
            TSID::from(200u64),
            OrganizationEntityType::Client,
            "system",
        );

        let saved_link = with_dummy_conn!(|conn| repo.link_entity(conn, &link))
            .await
            .unwrap();

        assert_eq!(saved_link.organization_id(), org_id);
        assert_eq!(saved_link.entity_id(), TSID::from(200u64));
        assert_eq!(saved_link.entity_type(), OrganizationEntityType::Client);
        assert_eq!(repo.links().len(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_link_entity_org_not_found() {
        use crate::domain::{OrganizationEntityType, OrganizationLink};

        let repo = InMemoryOrganizationRepository::new();
        let link = OrganizationLink::new(
            OrganizationId::new(TSID::from(999u64)),
            TSID::from(200u64),
            OrganizationEntityType::Client,
            "system",
        );

        let res = with_dummy_conn!(|conn| repo.link_entity(conn, &link)).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_in_memory_link_entity_already_exists() {
        use crate::domain::{OrganizationEntityType, OrganizationLink};

        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Teste", doc)
            .with_audit(AuditFields::created("admin"));

        with_dummy_conn!(|conn| repo.save(conn, &org))
            .await
            .unwrap();

        let link = OrganizationLink::new(
            org_id,
            TSID::from(200u64),
            OrganizationEntityType::Client,
            "system",
        );

        with_dummy_conn!(|conn| repo.link_entity(conn, &link))
            .await
            .unwrap();

        let duplicate = OrganizationLink::new(
            org_id,
            TSID::from(200u64),
            OrganizationEntityType::Client,
            "system",
        );

        let res = with_dummy_conn!(|conn| repo.link_entity(conn, &duplicate)).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), OrganizationBusinessErrors::already_exists());
    }

    #[tokio::test]
    async fn test_in_memory_unlink_entity_success() {
        use crate::domain::{OrganizationEntityType, OrganizationLink};

        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Teste", doc)
            .with_audit(AuditFields::created("admin"));

        with_dummy_conn!(|conn| repo.save(conn, &org))
            .await
            .unwrap();

        let link1 = OrganizationLink::new(
            org_id,
            TSID::from(200u64),
            OrganizationEntityType::Client,
            "system",
        );
        let link2 = OrganizationLink::new(
            org_id,
            TSID::from(300u64),
            OrganizationEntityType::Client,
            "system",
        );

        with_dummy_conn!(|conn| repo.link_entity(conn, &link1))
            .await
            .unwrap();
        with_dummy_conn!(|conn| repo.link_entity(conn, &link2))
            .await
            .unwrap();

        assert_eq!(repo.links().len(), 2);

        let res = with_dummy_conn!(|conn| {
            repo.unlink_entity(conn, TSID::from(200u64), OrganizationEntityType::Client)
        })
        .await;

        assert!(res.is_ok());
        let remaining = repo.links();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].entity_id(), TSID::from(300u64));
    }

    #[tokio::test]
    async fn test_in_memory_unlink_entity_not_found_is_noop() {
        use crate::domain::OrganizationEntityType;

        let repo = InMemoryOrganizationRepository::new();
        let res = with_dummy_conn!(|conn| {
            repo.unlink_entity(conn, TSID::from(999u64), OrganizationEntityType::Client)
        })
        .await;

        assert!(res.is_ok());
        assert_eq!(repo.links().len(), 0);
    }
}
