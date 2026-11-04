use crate::domain::organization::Organization;
use async_trait::async_trait;
use shared::entities::BusinessError;
use shared::types::page::PageRequest;
use sqlx::{Postgres, Transaction};
use tsid::TSID;

/// Trait que define o contrato de persistência para a entidade `Organization`.
#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    /// Salva uma organização na base de dados.
    async fn save(&self, organization: &Organization) -> Result<Organization, BusinessError>;

    async fn find_by_id(&self, id: TSID) -> Result<Organization, BusinessError>;

    /// Remove logicamente uma organização pelo seu identificador TSID.
    async fn delete(&self, id: TSID, updated_by: &str) -> Result<(), BusinessError>;

    /// Consulta organizações aplicando filtro opcional por nome e paginação por PageRequest.
    async fn find_all(
        &self,
        name: Option<&str>,
        page_request: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError>;

    /// Retorna a quantidade total de organizações que atendem ao filtro opcional por nome.
    async fn count(&self, name: Option<&str>) -> Result<u64, BusinessError>;

    // Transaction-aware variants keep tenant search_path and all writes in the same transaction.
    async fn save_in_tx(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        value: &Organization,
    ) -> Result<Organization, BusinessError> {
        self.save(value).await
    }
    async fn find_by_id_in_tx(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        id: TSID,
    ) -> Result<Organization, BusinessError> {
        self.find_by_id(id).await
    }
    async fn delete_in_tx(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError> {
        self.delete(id, updated_by).await
    }
    async fn find_all_in_tx(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        name: Option<&str>,
        page: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError> {
        self.find_all(name, page).await
    }
    async fn count_in_tx(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        name: Option<&str>,
    ) -> Result<u64, BusinessError> {
        self.count(name).await
    }
}
