use crate::domain::organization::{Organization, OrganizationId};
use crate::domain::organization_repository::OrganizationRepository;
use crate::errors::OrganizationBusinessErrors;
use async_trait::async_trait;
use shared::entities::BusinessError;
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
}
