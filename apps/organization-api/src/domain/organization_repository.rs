use crate::domain::organization::Organization;
use async_trait::async_trait;
use shared::entities::BusinessError;
use tsid::TSID;

/// Trait que define o contrato de persistência para a entidade `Organization`.
#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    /// Salva uma organização na base de dados.
    async fn save(
        &self,
        organization: &Organization,
    ) -> Result<Organization, BusinessError>;

    async fn find_by_id(&self, id: TSID) -> Result<Organization, BusinessError>;

    /// Remove logicamente uma organização pelo seu identificador TSID.
    async fn delete(&self, id: TSID, updated_by: &str) -> Result<(), BusinessError>;
}
