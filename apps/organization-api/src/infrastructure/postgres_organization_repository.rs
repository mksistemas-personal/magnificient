use crate::domain::organization::{Organization, OrganizationId};
use crate::domain::organization_repository::OrganizationRepository;
use crate::errors::OrganizationBusinessErrors;
use async_trait::async_trait;
use database::DatabaseConnection;
use shared::entities::{AuditFields, BusinessError, Country, Document, DocumentType, TSID};
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

        let query = r#"
            INSERT INTO organizations (
                id, name, document_value, document_type, document_country,
                deleted, created_at, updated_at, created_by, updated_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                document_value = EXCLUDED.document_value,
                document_type = EXCLUDED.document_type,
                document_country = EXCLUDED.document_country,
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
            SELECT id, name, document_value, document_type, document_country,
                   created_at, updated_at, created_by, updated_by
            FROM organizations
            WHERE id = $1 AND deleted = FALSE
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

        let id_raw: i64 = row_get_field(&row, "id")?;
        let name: String = row_get_field(&row, "name")?;
        let doc_value: String = row_get_field(&row, "document_value")?;
        let doc_type_str: String = row_get_field(&row, "document_type")?;
        let doc_country: Option<String> = row_get_field(&row, "document_country")?;
        let created_at: chrono::DateTime<chrono::Utc> = row_get_field(&row, "created_at")?;
        let updated_at: chrono::DateTime<chrono::Utc> = row_get_field(&row, "updated_at")?;
        let created_by: String = row_get_field(&row, "created_by")?;
        let updated_by: String = row_get_field(&row, "updated_by")?;

        let country = doc_country.as_deref().map(Country::from_code);
        let document_type = DocumentType::from_code(&doc_type_str, country);
        let document = Document::new(doc_value, document_type);
        let audit = AuditFields {
            created_at,
            updated_at,
            created_by,
            updated_by,
        };

        let organization = Organization::new(
            OrganizationId::new(TSID::from(id_raw as u64)),
            name,
            document,
        )
        .with_audit(audit);

        Ok(organization)
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
                SELECT id, name, document_value, document_type, document_country,
                       created_at, updated_at, created_by, updated_by
                FROM organizations
                WHERE deleted = FALSE AND name ILIKE $1
                ORDER BY name, id
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
                SELECT id, name, document_value, document_type, document_country,
                       created_at, updated_at, created_by, updated_by
                FROM organizations
                WHERE deleted = FALSE
                ORDER BY name, id
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
        for row in rows {
            let id_raw: i64 = row_get_field(&row, "id")?;
            let name: String = row_get_field(&row, "name")?;
            let doc_value: String = row_get_field(&row, "document_value")?;
            let doc_type_str: String = row_get_field(&row, "document_type")?;
            let doc_country: Option<String> = row_get_field(&row, "document_country")?;
            let created_at: chrono::DateTime<chrono::Utc> = row_get_field(&row, "created_at")?;
            let updated_at: chrono::DateTime<chrono::Utc> = row_get_field(&row, "updated_at")?;
            let created_by: String = row_get_field(&row, "created_by")?;
            let updated_by: String = row_get_field(&row, "updated_by")?;

            let country = doc_country.as_deref().map(Country::from_code);
            let document_type = DocumentType::from_code(&doc_type_str, country);
            let document = Document::new(doc_value, document_type);
            let audit = AuditFields {
                created_at,
                updated_at,
                created_by,
                updated_by,
            };

            let organization = Organization::new(
                OrganizationId::new(TSID::from(id_raw as u64)),
                name,
                document,
            )
            .with_audit(audit);

            organizations.push(organization);
        }

        Ok(organizations)
    }

    /// Executa a contagem de organizações que atendem ao filtro opcional por nome no PostgreSQL.
    async fn execute_count<'e, E>(
        executor: E,
        name: Option<&str>,
    ) -> Result<u64, BusinessError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let name_filter = name.map(|n| n.trim()).filter(|n| !n.is_empty());
        let count_res: Result<i64, _> = if let Some(filter) = name_filter {
            let pattern = format!("%{filter}%");
            let query = "SELECT COUNT(*) FROM organizations WHERE deleted = FALSE AND name ILIKE $1";
            sqlx::query_scalar(query)
                .bind(pattern)
                .fetch_one(executor)
                .await
        } else {
            let query = "SELECT COUNT(*) FROM organizations WHERE deleted = FALSE";
            sqlx::query_scalar(query)
                .fetch_one(executor)
                .await
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
