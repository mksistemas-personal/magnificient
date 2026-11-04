use crate::domain::organization::{Organization, OrganizationId};
use crate::domain::organization_repository::OrganizationRepository;
use crate::errors::OrganizationBusinessErrors;
use async_trait::async_trait;
use database::DatabaseConnection;
use shared::entities::{
    Address, AuditFields, BusinessError, Country, Document, DocumentType, TSID,
};
use shared::row_get_field;
use shared::types::page::PageRequest;
use sqlx::PgPool;
use tracing::error;

/// Implementação do repositório de `Organization` utilizando PostgreSQL e SQLx.
#[derive(Debug, Clone)]
pub struct PostgresOrganizationRepository {
    pool: PgPool,
}

impl PostgresOrganizationRepository {
    /// Cria uma nova instância a partir de um `PgPool` do SQLx.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Cria uma nova instância a partir de uma `DatabaseConnection`.
    pub fn from_connection(connection: &DatabaseConnection) -> Self {
        Self {
            pool: connection.pool().clone(),
        }
    }

    /// Retorna uma referência ao pool de conexões subjacente.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Salva a organização utilizando uma transação ativa do SQLx (`Transaction<'_, Postgres>`).
    pub async fn save_with_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        organization: &Organization,
    ) -> Result<Organization, BusinessError> {
        Self::execute_save(&mut **tx, organization).await
    }

    /// Busca uma organização pelo ID utilizando uma transação ativa do SQLx (`Transaction<'_, Postgres>`).
    pub async fn find_by_id_with_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: TSID,
    ) -> Result<Organization, BusinessError> {
        Self::execute_find_by_id(&mut **tx, id).await
    }

    /// Remove logicamente uma organização pelo ID utilizando uma transação ativa do SQLx (`Transaction<'_, Postgres>`).
    pub async fn delete_with_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError> {
        Self::execute_delete(&mut **tx, id, updated_by).await
    }

    /// Consulta organizações aplicando filtro opcional por nome e paginação por PageRequest utilizando uma transação ativa do SQLx.
    pub async fn find_all_with_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        name: Option<&str>,
        page_request: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError> {
        Self::execute_find_all(&mut **tx, name, page_request).await
    }

    /// Retorna a quantidade total de organizações que atendem ao filtro opcional por nome utilizando uma transação ativa do SQLx.
    pub async fn count_with_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        name: Option<&str>,
    ) -> Result<u64, BusinessError> {
        Self::execute_count(&mut **tx, name).await
    }

    /// Executa o comando de inserção/atualização no PostgreSQL utilizando qualquer executor SQLx compatível.
    async fn execute_save<'e, E>(
        executor: E,
        organization: &Organization,
    ) -> Result<Organization, BusinessError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let (created_at, updated_at, created_by, updated_by) = match &organization.audit {
            Some(audit) => (
                audit.created_at(),
                audit.updated_at(),
                audit.created_by().to_string(),
                audit.updated_by().to_string(),
            ),
            None => {
                let now = chrono::Utc::now();
                (now, now, "system".to_string(), "system".to_string())
            }
        };

        let id_raw = organization.id().number() as i64;
        let doc_country = organization
            .document()
            .country()
            .map(|country| country.code().to_string());
        let address_id = organization
            .address()
            .and_then(|addr| addr.id())
            .map(|id| id.number() as i64);

        let query = r#"
            INSERT INTO organizations (
                id, name, document_value, document_type, document_country,
                address_id, deleted, created_at, updated_at, created_by, updated_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                document_value = EXCLUDED.document_value,
                document_type = EXCLUDED.document_type,
                document_country = EXCLUDED.document_country,
                address_id = EXCLUDED.address_id,
                deleted = EXCLUDED.deleted,
                updated_at = EXCLUDED.updated_at,
                updated_by = EXCLUDED.updated_by
        "#;

        sqlx::query(query)
            .bind(id_raw)
            .bind(organization.name())
            .bind(organization.document().value())
            .bind(organization.document().document_type().code())
            .bind(doc_country)
            .bind(address_id)
            .bind(false)
            .bind(created_at)
            .bind(updated_at)
            .bind(created_by)
            .bind(updated_by)
            .execute(executor)
            .await
            .map_err(|err| {
                if let Some(db_err) = err.as_database_error() {
                    // 23505 = unique_violation no PostgreSQL
                    if db_err.code().as_deref() == Some("23505") {
                        return OrganizationBusinessErrors::document_already_in_use();
                    }
                }
                error!("Erro ao salvar organização no banco de dados: {:?}", err);
                BusinessError::internal(format!("Falha ao persistir organização: {err}"))
            })?;

        Ok(organization.clone())
    }

    /// Executa a consulta de busca por ID no PostgreSQL utilizando qualquer executor SQLx compatível.
    async fn execute_find_by_id<'e, E>(executor: E, id: TSID) -> Result<Organization, BusinessError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let id_raw = id.number() as i64;
        let query = r#"
            SELECT o.id, o.name, o.document_value, o.document_type, o.document_country,
                   o.created_at, o.updated_at, o.created_by, o.updated_by,
                   a.id AS addr_id, a.street, a.number, a.complement, a.neighborhood,
                   a.city, a.state, a.state_code, a.postal_code, a.country AS addr_country,
                   a.country_code AS addr_country_code, a.reference,
                   a.created_at AS addr_created_at, a.updated_at AS addr_updated_at,
                   a.created_by AS addr_created_by, a.updated_by AS addr_updated_by
            FROM organizations o
            LEFT JOIN addresses a ON o.address_id = a.id AND a.deleted = FALSE
            WHERE o.id = $1 AND o.deleted = FALSE
        "#;

        let row_opt = sqlx::query(query)
            .bind(id_raw)
            .fetch_optional(executor)
            .await
            .map_err(|err| {
                error!("Fail to get organization by id {}: {:?}", id, err);
                OrganizationBusinessErrors::not_found()
            })?;

        let row = match row_opt {
            Some(row) => row,
            None => return Err(OrganizationBusinessErrors::not_found()),
        };

        Self::map_row_to_organization(&row)
    }

    /// Executa o comando de exclusão lógica (soft delete) no PostgreSQL utilizando qualquer executor SQLx compatível.
    async fn execute_delete<'e, E>(
        executor: E,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let id_raw = id.number() as i64;
        let now = chrono::Utc::now();
        let query = r#"
            UPDATE organizations
            SET deleted = TRUE, updated_at = $2, updated_by = $3
            WHERE id = $1 AND deleted = FALSE
        "#;

        let result = sqlx::query(query)
            .bind(id_raw)
            .bind(now)
            .bind(updated_by)
            .execute(executor)
            .await
            .map_err(|err| {
                error!("Fail to delete organization by id {}: {:?}", id, err);
                BusinessError::internal(format!("Falha ao deletar organização: {err}"))
            })?;

        if result.rows_affected() == 0 {
            return Err(OrganizationBusinessErrors::not_found());
        }

        Ok(())
    }

    /// Executa a consulta paginada com filtro opcional por nome no PostgreSQL utilizando qualquer executor SQLx compatível.
    async fn execute_find_all<'e, E>(
        executor: E,
        name: Option<&str>,
        page_request: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let name_filter = name.map(|n| n.trim()).filter(|n| !n.is_empty());
        let limit = page_request.limit() as i64;
        let offset = page_request.offset() as i64;

        let rows = if let Some(filter) = name_filter {
            let pattern = format!("%{filter}%");
            let query = r#"
                SELECT o.id, o.name, o.document_value, o.document_type, o.document_country,
                       o.created_at, o.updated_at, o.created_by, o.updated_by,
                       a.id AS addr_id, a.street, a.number, a.complement, a.neighborhood,
                       a.city, a.state, a.state_code, a.postal_code, a.country AS addr_country,
                       a.country_code AS addr_country_code, a.reference,
                       a.created_at AS addr_created_at, a.updated_at AS addr_updated_at,
                       a.created_by AS addr_created_by, a.updated_by AS addr_updated_by
                FROM organizations o
                LEFT JOIN addresses a ON o.address_id = a.id AND a.deleted = FALSE
                WHERE o.deleted = FALSE AND o.name ILIKE $1
                ORDER BY o.name, o.id
                LIMIT $2 OFFSET $3
            "#;
            sqlx::query(query)
                .bind(pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(executor)
                .await
        } else {
            let query = r#"
                SELECT o.id, o.name, o.document_value, o.document_type, o.document_country,
                       o.created_at, o.updated_at, o.created_by, o.updated_by,
                       a.id AS addr_id, a.street, a.number, a.complement, a.neighborhood,
                       a.city, a.state, a.state_code, a.postal_code, a.country AS addr_country,
                       a.country_code AS addr_country_code, a.reference,
                       a.created_at AS addr_created_at, a.updated_at AS addr_updated_at,
                       a.created_by AS addr_created_by, a.updated_by AS addr_updated_by
                FROM organizations o
                LEFT JOIN addresses a ON o.address_id = a.id AND a.deleted = FALSE
                WHERE o.deleted = FALSE
                ORDER BY o.name, o.id
                LIMIT $1 OFFSET $2
            "#;
            sqlx::query(query)
                .bind(limit)
                .bind(offset)
                .fetch_all(executor)
                .await
        };

        let rows = rows.map_err(|err| {
            error!("Fail to list organizations: {:?}", err);
            BusinessError::internal(format!("Falha ao consultar organizações: {err}"))
        })?;

        let mut organizations = Vec::with_capacity(rows.len());
        for row in &rows {
            organizations.push(Self::map_row_to_organization(row)?);
        }

        Ok(organizations)
    }

    fn map_row_to_organization(row: &sqlx::postgres::PgRow) -> Result<Organization, BusinessError> {
        let id_raw: i64 = row_get_field(row, "id")?;
        let name: String = row_get_field(row, "name")?;
        let doc_value: String = row_get_field(row, "document_value")?;
        let doc_type_str: String = row_get_field(row, "document_type")?;
        let doc_country: Option<String> = row_get_field(row, "document_country")?;
        let created_at: chrono::DateTime<chrono::Utc> = row_get_field(row, "created_at")?;
        let updated_at: chrono::DateTime<chrono::Utc> = row_get_field(row, "updated_at")?;
        let created_by: String = row_get_field(row, "created_by")?;
        let updated_by: String = row_get_field(row, "updated_by")?;

        let country = doc_country.as_deref().map(Country::from_code);
        let document_type = DocumentType::from_code(&doc_type_str, country);
        let document = Document::new(doc_value, document_type);
        let audit = AuditFields {
            created_at,
            updated_at,
            created_by,
            updated_by,
        };

        let mut organization = Organization::new(
            OrganizationId::new(TSID::from(id_raw as u64)),
            name,
            document,
        )
        .with_audit(audit);

        let addr_id: Option<i64> = row_get_field(row, "addr_id").ok().flatten();
        if let Some(addr_id_val) = addr_id {
            let street: String = row_get_field(row, "street").unwrap_or_default();
            let number: String = row_get_field(row, "number").unwrap_or_default();
            let complement: Option<String> = row_get_field(row, "complement").ok().flatten();
            let neighborhood: Option<String> = row_get_field(row, "neighborhood").ok().flatten();
            let city: String = row_get_field(row, "city").unwrap_or_default();
            let state: Option<String> = row_get_field(row, "state").ok().flatten();
            let state_code: Option<String> = row_get_field(row, "state_code").ok().flatten();
            let postal_code: String = row_get_field(row, "postal_code").unwrap_or_default();
            let addr_country: Option<String> = row_get_field(row, "addr_country").ok().flatten();
            let addr_country_code: Option<String> =
                row_get_field(row, "addr_country_code").ok().flatten();
            let reference: Option<String> = row_get_field(row, "reference").ok().flatten();
            let addr_created_at: Option<chrono::DateTime<chrono::Utc>> =
                row_get_field(row, "addr_created_at").ok().flatten();
            let addr_updated_at: Option<chrono::DateTime<chrono::Utc>> =
                row_get_field(row, "addr_updated_at").ok().flatten();
            let addr_created_by: Option<String> =
                row_get_field(row, "addr_created_by").ok().flatten();
            let addr_updated_by: Option<String> =
                row_get_field(row, "addr_updated_by").ok().flatten();

            let mut address = Address::new(
                street,
                number,
                complement.unwrap_or_default(),
                neighborhood.unwrap_or_default(),
                city,
                state.unwrap_or_default(),
                state_code.unwrap_or_default(),
                postal_code,
                addr_country.unwrap_or_default(),
                addr_country_code.unwrap_or_default(),
                reference.unwrap_or_default(),
            );
            address.set_id(TSID::from(addr_id_val as u64));
            if let (Some(c_at), Some(u_at), Some(c_by), Some(u_by)) = (
                addr_created_at,
                addr_updated_at,
                addr_created_by,
                addr_updated_by,
            ) {
                address.set_audit(Some(AuditFields {
                    created_at: c_at,
                    updated_at: u_at,
                    created_by: c_by,
                    updated_by: u_by,
                }));
            }
            organization = organization.with_address(address);
        }

        Ok(organization)
    }

    /// Executa a contagem de organizações que atendem ao filtro opcional por nome no PostgreSQL.
    async fn execute_count<'e, E>(executor: E, name: Option<&str>) -> Result<u64, BusinessError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let name_filter = name.map(|n| n.trim()).filter(|n| !n.is_empty());
        let count_res: Result<i64, _> = if let Some(filter) = name_filter {
            let pattern = format!("%{filter}%");
            let query =
                "SELECT COUNT(*) FROM organizations WHERE deleted = FALSE AND name ILIKE $1";
            sqlx::query_scalar(query)
                .bind(pattern)
                .fetch_one(executor)
                .await
        } else {
            let query = "SELECT COUNT(*) FROM organizations WHERE deleted = FALSE";
            sqlx::query_scalar(query).fetch_one(executor).await
        };

        let total = count_res.map_err(|err| {
            error!("Fail to count organizations: {:?}", err);
            BusinessError::internal(format!("Falha ao contar organizações: {err}"))
        })?;

        Ok(total as u64)
    }
}

impl From<PgPool> for PostgresOrganizationRepository {
    fn from(pool: PgPool) -> Self {
        Self::new(pool)
    }
}

impl From<DatabaseConnection> for PostgresOrganizationRepository {
    fn from(connection: DatabaseConnection) -> Self {
        Self::new(connection.into_inner())
    }
}

impl From<&DatabaseConnection> for PostgresOrganizationRepository {
    fn from(connection: &DatabaseConnection) -> Self {
        Self::from_connection(connection)
    }
}

#[async_trait]
impl OrganizationRepository for PostgresOrganizationRepository {
    async fn save(&self, organization: &Organization) -> Result<Organization, BusinessError> {
        Self::execute_save(&self.pool, organization).await
    }

    async fn find_by_id(&self, id: TSID) -> Result<Organization, BusinessError> {
        Self::execute_find_by_id(&self.pool, id).await
    }

    async fn delete(&self, id: TSID, updated_by: &str) -> Result<(), BusinessError> {
        Self::execute_delete(&self.pool, id, updated_by).await
    }

    async fn find_all(
        &self,
        name: Option<&str>,
        page_request: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError> {
        Self::execute_find_all(&self.pool, name, page_request).await
    }

    async fn count(&self, name: Option<&str>) -> Result<u64, BusinessError> {
        Self::execute_count(&self.pool, name).await
    }

    async fn save_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        value: &Organization,
    ) -> Result<Organization, BusinessError> {
        Self::save_with_tx(tx, value).await
    }
    async fn find_by_id_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: TSID,
    ) -> Result<Organization, BusinessError> {
        Self::find_by_id_with_tx(tx, id).await
    }
    async fn delete_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError> {
        Self::delete_with_tx(tx, id, updated_by).await
    }
    async fn find_all_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        name: Option<&str>,
        page: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError> {
        Self::find_all_with_tx(tx, name, page).await
    }
    async fn count_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        name: Option<&str>,
    ) -> Result<u64, BusinessError> {
        Self::count_with_tx(tx, name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::DatabaseConnectionConfig;

    #[tokio::test]
    async fn test_postgres_repository_initialization_and_conversions() {
        let config = DatabaseConnectionConfig::default();
        let db = DatabaseConnection::connect_lazy_with_config(&config);

        let repo_from_conn = PostgresOrganizationRepository::from_connection(&db);
        assert!(!repo_from_conn.pool().is_closed());

        let repo_from_ref: PostgresOrganizationRepository = (&db).into();
        assert!(!repo_from_ref.pool().is_closed());

        let pool = db.pool().clone();
        let repo_from_pool: PostgresOrganizationRepository = pool.into();
        assert!(!repo_from_pool.pool().is_closed());
    }
}
