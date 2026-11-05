use crate::domain::organization::{Organization, OrganizationId};
use crate::domain::organization_repository::OrganizationRepository;
use crate::errors::OrganizationBusinessErrors;
use async_trait::async_trait;
use database::DatabaseConnection;
use shared::entities::{
    Address, AuditFields, BusinessError, Country, Document, DocumentType, TSID,
};
use shared::types::page::PageRequest;
use sqlx::PgPool;
use tracing::error;

const ORGANIZATION_SELECT: &str = r#"
    SELECT o.id, o.name, o.document_value, o.document_type, o.document_country,
           o.created_at, o.updated_at, o.created_by, o.updated_by,
           a.id AS addr_id, a.street AS addr_street, a.number AS addr_number,
           a.complement AS addr_complement, a.neighborhood AS addr_neighborhood,
           a.city AS addr_city, a.state AS addr_state, a.state_code AS addr_state_code,
           a.postal_code AS addr_postal_code, a.country AS addr_country,
           a.country_code AS addr_country_code, a.reference AS addr_reference,
           a.created_at AS addr_created_at, a.updated_at AS addr_updated_at,
           a.created_by AS addr_created_by, a.updated_by AS addr_updated_by
    FROM organizations o
    LEFT JOIN addresses a ON o.address_id = a.id AND a.deleted = FALSE
    WHERE o.deleted = FALSE
"#;

#[derive(sqlx::FromRow)]
struct OrganizationRow {
    id: i64,
    name: String,
    document_value: String,
    document_type: String,
    document_country: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    created_by: String,
    updated_by: String,
    addr_id: Option<i64>,
    addr_street: Option<String>,
    addr_number: Option<String>,
    addr_complement: Option<String>,
    addr_neighborhood: Option<String>,
    addr_city: Option<String>,
    addr_state: Option<String>,
    addr_state_code: Option<String>,
    addr_postal_code: Option<String>,
    addr_country: Option<String>,
    addr_country_code: Option<String>,
    addr_reference: Option<String>,
    addr_created_at: Option<chrono::DateTime<chrono::Utc>>,
    addr_updated_at: Option<chrono::DateTime<chrono::Utc>>,
    addr_created_by: Option<String>,
    addr_updated_by: Option<String>,
}

impl TryFrom<OrganizationRow> for Organization {
    type Error = BusinessError;

    fn try_from(row: OrganizationRow) -> Result<Self, Self::Error> {
        let country = row.document_country.as_deref().map(Country::from_code);
        let document_type = DocumentType::from_code(&row.document_type, country);
        let document = Document::new(row.document_value, document_type);
        let audit = AuditFields {
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
            updated_by: row.updated_by,
        };
        let mut organization = Organization::new(
            OrganizationId::new(TSID::from(row.id as u64)),
            row.name,
            document,
        )
        .with_audit(audit);

        if let Some(address_id) = row.addr_id {
            let address = Address::new(
                required_address_field(row.addr_street, "street")?,
                required_address_field(row.addr_number, "number")?,
                row.addr_complement.unwrap_or_default(),
                row.addr_neighborhood.unwrap_or_default(),
                required_address_field(row.addr_city, "city")?,
                row.addr_state.unwrap_or_default(),
                row.addr_state_code.unwrap_or_default(),
                required_address_field(row.addr_postal_code, "postal_code")?,
                row.addr_country.unwrap_or_default(),
                row.addr_country_code.unwrap_or_default(),
                row.addr_reference.unwrap_or_default(),
            )
            .with_id(TSID::from(address_id as u64))
            .with_audit(AuditFields {
                created_at: required_address_field(row.addr_created_at, "created_at")?,
                updated_at: required_address_field(row.addr_updated_at, "updated_at")?,
                created_by: required_address_field(row.addr_created_by, "created_by")?,
                updated_by: required_address_field(row.addr_updated_by, "updated_by")?,
            });
            organization = organization.with_address(address);
        }

        Ok(organization)
    }
}

fn required_address_field<T>(value: Option<T>, field: &str) -> Result<T, BusinessError> {
    value.ok_or_else(|| {
        BusinessError::internal(format!(
            "Required field missing from associated address: {field}"
        ))
    })
}

/// PostgreSQL and SQLx implementation of the `Organization` repository.
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

    /// Returns a reference to the underlying connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
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
                error!("Failed to save organization to the database: {:?}", err);
                BusinessError::internal(format!("Failed to persist organization: {err}"))
            })?;

        Ok(organization.clone())
    }

    /// Executa a consulta de busca por ID no PostgreSQL utilizando qualquer executor SQLx compatível.
    async fn execute_find_by_id<'e, E>(executor: E, id: TSID) -> Result<Organization, BusinessError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let id_raw = id.number() as i64;
        let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new(ORGANIZATION_SELECT);
        let row_opt = query
            .push(" AND o.id = ")
            .push_bind(id_raw)
            .build_query_as::<OrganizationRow>()
            .fetch_optional(executor)
            .await
            .map_err(|err| {
                error!("Failed to fetch organization by ID {}: {:?}", id, err);
                BusinessError::internal(format!("Failed to query organization: {err}"))
            })?;

        let row = match row_opt {
            Some(row) => row,
            None => return Err(OrganizationBusinessErrors::not_found()),
        };

        row.try_into()
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
                error!("Failed to delete organization by ID {}: {:?}", id, err);
                BusinessError::internal(format!("Failed to delete organization: {err}"))
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

        let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new(ORGANIZATION_SELECT);
        if let Some(filter) = name_filter {
            query
                .push(" AND o.name ILIKE ")
                .push_bind(format!("%{filter}%"));
        }

        let rows = query
            .push(" ORDER BY o.name, o.id LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset)
            .build_query_as::<OrganizationRow>()
            .fetch_all(executor)
            .await
            .map_err(|err| {
                error!("Failed to list organizations: {:?}", err);
                BusinessError::internal(format!("Failed to query organizations: {err}"))
            })?;

        rows.into_iter().map(TryInto::try_into).collect()
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
            BusinessError::internal(format!("Failed to count organizations: {err}"))
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
        Self::execute_save(&mut **tx, value).await
    }
    async fn find_by_id_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: TSID,
    ) -> Result<Organization, BusinessError> {
        Self::execute_find_by_id(&mut **tx, id).await
    }
    async fn delete_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError> {
        Self::execute_delete(&mut **tx, id, updated_by).await
    }
    async fn find_all_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        name: Option<&str>,
        page: &PageRequest,
    ) -> Result<Vec<Organization>, BusinessError> {
        Self::execute_find_all(&mut **tx, name, page).await
    }
    async fn count_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        name: Option<&str>,
    ) -> Result<u64, BusinessError> {
        Self::execute_count(&mut **tx, name).await
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
