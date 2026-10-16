use crate::domain::organization::{Organization, OrganizationId};
use crate::domain::organization_repository::OrganizationRepository;
use crate::errors::OrganizationBusinessErrors;
use async_trait::async_trait;
use database::DatabaseConnection;
use shared::entities::{AuditFields, BusinessError, Country, Document, DocumentType, TSID};
use shared::row_get_field;
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

    /// Executa o comando de inserção/atualização no PostgreSQL utilizando qualquer executor SQLx compatível.
    async fn execute_save<'e, E>(
        executor: E,
        organization: &Organization,
    ) -> Result<Organization, BusinessError>
    where
        E: sqlx::Executor<'e, Database=sqlx::Postgres>,
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
    async fn execute_find_by_id<'e, E>(
        executor: E,
        id: TSID,
    ) -> Result<Organization, BusinessError>
    where
        E: sqlx::Executor<'e, Database=sqlx::Postgres>,
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

        let country = doc_country.map(|c| match c.to_uppercase().as_str() {
            "BR" => Country::Brazil,
            "AR" => Country::Argentina,
            other => Country::Other(other.to_string()),
        });

        let document_type = match doc_type_str.to_uppercase().as_str() {
            "CNPJ" => DocumentType::Cnpj,
            "CPF" => DocumentType::Cpf,
            "CUIT" => DocumentType::Cuit,
            "CUIL" => DocumentType::Cuil,
            "DNI" => DocumentType::Dni,
            "PASSPORT" | "PASSAPORTE" => DocumentType::Passport,
            other => DocumentType::Other {
                name: other.to_string(),
                country,
            },
        };

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
    async fn save(
        &self,
        organization: &Organization,
    ) -> Result<Organization, BusinessError> {
        Self::execute_save(&self.pool, organization).await
    }

    async fn find_by_id(&self, id: TSID) -> Result<Organization, BusinessError> {
        Self::execute_find_by_id(&self.pool, id).await
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
