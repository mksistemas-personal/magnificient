use crate::domain::organization::Organization;
use async_trait::async_trait;
use shared::entities::BusinessError;
use shared::types::page::PageRequest;
use sqlx::{Postgres, Transaction};
use tsid::TSID;

/// Trait que define o contrato de persistência para a entidade `Organization`.
#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    /// Saves an organization to the database.
    async fn save(&self, organization: &Organization) -> Result<Organization, BusinessError>;

    async fn find_by_id(&self, id: TSID) -> Result<Organization, BusinessError>;

    /// Soft-deletes an organization by its TSID.
    async fn delete(&self, id: TSID, updated_by: &str) -> Result<(), BusinessError>;

    /// Consulta organizações aplicando filtro opcional por nome e paginação por PageRequest.
    async fn find_all(
        &self,
        name: Option<&str>,
        page_request: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError>;

    /// Returns the total number of organizations matching the optional name filter.
    async fn count(&self, name: Option<&str>) -> Result<u64, BusinessError>;

    // Implementations must explicitly state how transaction-aware operations behave.
    async fn save_in_tx(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        value: &Organization,
    ) -> Result<Organization, BusinessError>;
    async fn find_by_id_in_tx(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        id: TSID,
    ) -> Result<Organization, BusinessError>;
    async fn delete_in_tx(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError>;
    async fn find_all_in_tx(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        name: Option<&str>,
        page: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError>;
    async fn count_in_tx(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        name: Option<&str>,
    ) -> Result<u64, BusinessError>;
}
