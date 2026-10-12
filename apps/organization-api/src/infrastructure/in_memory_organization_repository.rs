use async_trait::async_trait;
use crate::application::errors::OrganizationBusinessErrors;
use crate::domain::organization::{Organization, OrganizationId};
use crate::domain::organization_repository::OrganizationRepository;
use shared::entities::BusinessError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
}
