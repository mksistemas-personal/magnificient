use crate::connection::DatabaseConnection;
use crate::error::DatabaseError;
use shared::entities::BusinessError;
use sqlx::{PgPool, Postgres, Transaction};
use std::future::Future;
use tracing::{debug, error};

/// Gerenciador de Unidade de Trabalho (Unit of Work) para controle de transações no PostgreSQL via SQLx.
///
/// Encapsula o ciclo de vida transacional (`begin`, `commit`, `rollback`) através de funções de
/// ordem superior (`run_tx`), garantindo atomicidade e isolamento de falhas de forma idiomática e reutilizável.
#[derive(Debug, Clone)]
pub struct UnitOfWork {
    pool: PgPool,
    schema: Option<String>,
}

impl UnitOfWork {
    /// Cria uma nova instância de `UnitOfWork` a partir de um `PgPool`.
    pub fn new(pool: PgPool) -> Self {
        Self { pool, schema: None }
    }

    /// Returns a unit of work that sets a tenant schema for each transaction.
    /// The schema is treated as an SQL identifier and validated before use.
    pub fn with_schema(mut self, schema: impl Into<String>) -> Result<Self, DatabaseError> {
        let schema = schema.into();
        if schema.is_empty()
            || schema.len() > 63
            || !schema
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            || !schema
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(DatabaseError::ConnectionError(sqlx::Error::Protocol(
                "Invalid PostgreSQL schema identifier".into(),
            )));
        }
        self.schema = Some(schema);
        Ok(self)
    }

    pub fn tenant(&self) -> Option<&str> {
        self.schema.as_deref()
    }

    /// Cria uma nova instância de `UnitOfWork` a partir de uma `DatabaseConnection`.
    pub fn from_connection(db: &DatabaseConnection) -> Self {
        Self {
            pool: db.pool().clone(),
            schema: None,
        }
    }

    /// Retorna uma referência ao pool de conexões subjacente.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Inicia uma nova transação no PostgreSQL.
    pub async fn begin(&self) -> Result<Transaction<'static, Postgres>, DatabaseError> {
        self.pool
            .begin()
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    /// Executa uma operação assíncrona dentro de um bloco transacional atômico.
    ///
    /// Se a operação retornar `Ok(T)`, a transação é automaticamente confirmada (`commit`).
    /// Se a operação retornar `Err(E)` ou ocorrer cancelamento, a transação é revertida (`rollback`).
    pub async fn run_tx<F, Fut, T>(&self, operation: F) -> Result<T, BusinessError>
    where
        F: FnOnce(Transaction<'static, Postgres>) -> Fut,
        Fut: Future<Output = Result<(T, Transaction<'static, Postgres>), BusinessError>>,
    {
        debug!("Iniciando transação na Unidade de Trabalho (Unit of Work)...");
        let mut tx = self.pool.begin().await.map_err(|err| {
            error!("Falha ao abrir transação no banco de dados: {:?}", err);
            BusinessError::internal(format!("Falha ao abrir transação no banco de dados: {err}"))
        })?;

        if let Some(schema) = &self.schema {
            // set_config accepts a value parameter, avoiding SQL identifier interpolation.
            let search_path = format!("\"{schema}\"");
            if let Err(err) = sqlx::query("SELECT set_config('search_path', $1, true)")
                .bind(search_path)
                .execute(&mut *tx)
                .await
            {
                let _ = tx.rollback().await;
                return Err(BusinessError::internal(format!(
                    "Falha ao selecionar schema do tenant: {err}"
                )));
            }
            let schema_exists: bool = match sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = $1)",
            )
            .bind(schema)
            .fetch_one(&mut *tx)
            .await
            {
                Ok(value) => value,
                Err(err) => {
                    let _ = tx.rollback().await;
                    return Err(BusinessError::internal(format!(
                        "Falha ao validar schema do tenant: {err}"
                    )));
                }
            };
            if !schema_exists {
                let _ = tx.rollback().await;
                return Err(BusinessError::not_found(format!(
                    "Schema do tenant '{schema}' não encontrado"
                )));
            }
        }

        match operation(tx).await {
            Ok((result, tx)) => {
                debug!("Confirmando (commit) transação da Unidade de Trabalho...");
                tx.commit().await.map_err(|err| {
                    error!("Falha ao confirmar (commit) transação: {:?}", err);
                    BusinessError::internal(format!(
                        "Falha ao confirmar transação no banco de dados: {err}"
                    ))
                })?;
                debug!("Transação confirmada com sucesso.");
                Ok(result)
            }
            Err(err) => {
                // `Transaction` rolls back on drop when the operation fails.
                debug!("Revertendo (rollback) transação devido a erro: {:?}", err);
                Err(err)
            }
        }
    }

    /// Executa uma operação assíncrona gerenciando a transação de forma opcional.
    ///
    /// Se `uow` for `Some(&UnitOfWork)`, a operação é executada dentro de uma transação via `run_tx`.
    /// Se `uow` for `None`, a operação é executada diretamente fora de uma transação.
    pub async fn execute_maybe_tx<F, Fut, T>(
        uow: Option<&UnitOfWork>,
        operation: F,
    ) -> Result<T, BusinessError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, BusinessError>> + 'static,
    {
        match uow {
            Some(u) => {
                u.run_tx(|tx| async move { Ok((operation().await?, tx)) })
                    .await
            }
            None => operation().await,
        }
    }
}

impl From<PgPool> for UnitOfWork {
    fn from(pool: PgPool) -> Self {
        Self::new(pool)
    }
}

impl From<&DatabaseConnection> for UnitOfWork {
    fn from(db: &DatabaseConnection) -> Self {
        Self::from_connection(db)
    }
}

impl From<DatabaseConnection> for UnitOfWork {
    fn from(db: DatabaseConnection) -> Self {
        Self::new(db.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConnectionConfig;

    #[tokio::test]
    async fn test_unit_of_work_creation_and_pool() {
        let config = DatabaseConnectionConfig::default();
        let db = DatabaseConnection::connect_lazy_with_config(&config);
        let uow = UnitOfWork::from_connection(&db);

        assert!(!uow.pool().is_closed());

        let uow_from_ref: UnitOfWork = (&db).into();
        assert!(!uow_from_ref.pool().is_closed());

        let uow_from_pool: UnitOfWork = db.pool().clone().into();
        assert!(!uow_from_pool.pool().is_closed());
    }

    #[tokio::test]
    async fn test_unit_of_work_execute_maybe_tx_none() {
        let result: Result<&str, BusinessError> =
            UnitOfWork::execute_maybe_tx(None, || async { Ok("success_without_uow") }).await;

        assert_eq!(result.unwrap(), "success_without_uow");
    }
}
