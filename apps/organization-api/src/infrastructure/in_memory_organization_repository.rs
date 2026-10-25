use crate::domain::organization::{Organization, OrganizationId};
use crate::domain::organization_repository::OrganizationRepository;
use crate::errors::OrganizationBusinessErrors;
use async_trait::async_trait;
use shared::entities::BusinessError;
use shared::types::page::PageRequest;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tsid::TSID;

/// Implementação em memória do repositório de `Organization`, ideal para testes unitários.
#[derive(Debug, Clone, Default)]
pub struct InMemoryOrganizationRepository {
    items: Arc<RwLock<HashMap<OrganizationId, Organization>>>,
}

impl InMemoryOrganizationRepository {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
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
}

#[async_trait]
impl OrganizationRepository for InMemoryOrganizationRepository {
    async fn save(
        &self,
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

    async fn find_by_id(&self, id: TSID) -> Result<Organization, BusinessError> {
        let org_id = OrganizationId::new(id);
        self.items
            .read()
            .unwrap()
            .get(&org_id)
            .cloned()
            .ok_or_else(OrganizationBusinessErrors::not_found)
    }

    async fn delete(&self, id: TSID, _updated_by: &str) -> Result<(), BusinessError> {
        let org_id = OrganizationId::new(id);
        let mut map = self.items.write().unwrap();
        if map.remove(&org_id).is_none() {
            return Err(OrganizationBusinessErrors::not_found());
        }
        Ok(())
    }

    async fn find_all(
        &self,
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

        list.sort_by(|a, b| a.id().cmp(&b.id()));

        let offset = page_request.offset() as usize;
        let limit = page_request.limit() as usize;

        let result = list.into_iter().skip(offset).take(limit).collect();
        Ok(result)
    }

    async fn count(&self, name: Option<&str>) -> Result<u64, BusinessError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::entities::{AuditFields, Document, DocumentType};

    #[tokio::test]
    async fn test_in_memory_find_by_id_success() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Empresa Teste", doc)
            .with_audit(AuditFields::created("admin"));

        repo.save(&org).await.unwrap();

        let found = OrganizationRepository::find_by_id(&repo, TSID::from(100u64))
            .await
            .unwrap();
        assert_eq!(found.id(), org_id);
        assert_eq!(found.name(), "Empresa Teste");
    }

    #[tokio::test]
    async fn test_in_memory_find_by_id_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let result = OrganizationRepository::find_by_id(&repo, TSID::from(999u64)).await;

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

        repo.save(&org).await.unwrap();

        let delete_res = OrganizationRepository::delete(&repo, TSID::from(100u64), "admin_delete").await;
        assert!(delete_res.is_ok());

        let find_res = OrganizationRepository::find_by_id(&repo, TSID::from(100u64)).await;
        assert!(find_res.is_err());
        assert_eq!(find_res.unwrap_err(), OrganizationBusinessErrors::not_found());
    }

    #[tokio::test]
    async fn test_in_memory_delete_not_found() {
        let repo = InMemoryOrganizationRepository::new();
        let result = OrganizationRepository::delete(&repo, TSID::from(999u64), "admin_delete").await;

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
            let org = Organization::new(org_id, name, doc)
                .with_audit(AuditFields::created("admin"));
            repo.save(&org).await.unwrap();
        }

        // Test filter by name "Alfa"
        let page_req = PageRequest::new(1, 10);
        let result = OrganizationRepository::find_all(&repo, Some("Alfa"), &page_req).await.unwrap();
        assert_eq!(result.len(), 5);
        for org in &result {
            assert!(org.name().contains("Alfa"));
        }

        // Test pagination: page 1, size 3
        let page1 = PageRequest::new(1, 3);
        let result_p1 = OrganizationRepository::find_all(&repo, None, &page1).await.unwrap();
        assert_eq!(result_p1.len(), 3);

        // Test pagination: page 2, size 3
        let page2 = PageRequest::new(2, 3);
        let result_p2 = OrganizationRepository::find_all(&repo, None, &page2).await.unwrap();
        assert_eq!(result_p2.len(), 3);
        assert_ne!(result_p1[0].id(), result_p2[0].id());

        // Test pagination: page 4, size 3 (should have 1 item out of 10)
        let page4 = PageRequest::new(4, 3);
        let result_p4 = OrganizationRepository::find_all(&repo, None, &page4).await.unwrap();
        assert_eq!(result_p4.len(), 1);

        // Test count
        let total_all = OrganizationRepository::count(&repo, None).await.unwrap();
        assert_eq!(total_all, 10);

        let total_alfa = OrganizationRepository::count(&repo, Some("Alfa")).await.unwrap();
        assert_eq!(total_alfa, 5);
    }
}
