use async_trait::async_trait;
use crate::domain::organization::Organization;
use shared::entities::BusinessError;

/// Trait que define o contrato de persistência para a entidade `Organization`.
#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    /// Salva uma organização na base de dados.
    async fn save(
        &self,
        organization: &Organization,
    ) -> Result<Organization, BusinessError>;
}
