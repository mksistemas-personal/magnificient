use crate::domain::organization::Organization;
use async_trait::async_trait;
use shared::entities::BusinessError;
use shared::types::page::PageRequest;
use sqlx::PgConnection;
use tsid::TSID;

/// Trait que define o contrato de persistência para a entidade `Organization`.
///
/// Aceita `&mut PgConnection` diretamente como executor, permitindo usar
/// tanto pool quanto transação sem duplicar a assinatura de cada método.
#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    /// Saves an organization to the database.
    async fn save(
        &self,
        conn: &mut PgConnection,
        organization: &Organization,
    ) -> Result<Organization, BusinessError>;

    async fn find_by_id(
        &self,
        conn: &mut PgConnection,
        id: TSID,
    ) -> Result<Organization, BusinessError>;

    /// Soft-deletes an organization by its TSID.
    async fn delete(
        &self,
        conn: &mut PgConnection,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError>;

    /// Consulta organizações aplicando filtro opcional por nome e paginação por PageRequest.
    async fn find_all(
        &self,
        conn: &mut PgConnection,
        name: Option<&str>,
        page_request: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError>;

    /// Returns the total number of organizations matching the optional name filter.
    async fn count(
        &self,
        conn: &mut PgConnection,
        name: Option<&str>,
    ) -> Result<u64, BusinessError>;
}
