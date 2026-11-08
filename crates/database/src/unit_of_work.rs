use crate::connection::DatabaseConnection;
use crate::error::DatabaseError;
use shared::entities::BusinessError;
use sqlx::{PgPool, Postgres, Transaction};
use std::future::Future;
use std::pin::Pin;
use tracing::{debug, error};

type TransactionOperationFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, BusinessError>> + Send + 'a>>;

/// Gerenciador de Unidade de Trabalho (Unit of Work) para controle de transações no PostgreSQL via SQLx.
///
/// Encapsula o ciclo de vida transacional (`begin`, `commit`, `rollback`) através de funções de
/// ordem superior (`transaction`), garantindo atomicidade e isolamento de falhas de forma idiomática e reutilizável.
#[derive(Debug, Clone)]
pub struct UnitOfWork {
    pool: PgPool,
    schema: Option<String>,
}

impl UnitOfWork {
    /// Creates a `UnitOfWork` from a `PgPool`.
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

    /// Creates a `UnitOfWork` from a `DatabaseConnection`.
    pub fn from_connection(db: &DatabaseConnection) -> Self {
        Self {
            pool: db.pool().clone(),
            schema: None,
        }
    }

    /// Returns a reference to the underlying connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Runs an asynchronous operation inside an atomic transaction.
    ///
    /// If the operation returns `Ok(T)`, the transaction is committed automatically.
    /// If the operation returns `Err(E)` or is cancelled, the transaction is rolled back.
    pub async fn transaction<F, T>(&self, operation: F) -> Result<T, BusinessError>
    where
        F: for<'a> FnOnce(
            &'a mut Transaction<'static, Postgres>,
        ) -> TransactionOperationFuture<'a, T>,
    {
        debug!("Starting Unit of Work transaction...");
        let mut tx = self.pool.begin().await.map_err(|err| {
            error!("Failed to begin database transaction: {:?}", err);
            BusinessError::internal(format!("Failed to begin database transaction: {err}"))
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
                    "Failed to select tenant schema: {err}"
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
                        "Failed to validate tenant schema: {err}"
                    )));
                }
            };
            if !schema_exists {
                let _ = tx.rollback().await;
                return Err(BusinessError::not_found(format!(
                    "Tenant schema '{schema}' not found"
                )));
            }
        }

        match operation(&mut tx).await {
            Ok(result) => {
                debug!("Committing Unit of Work transaction...");
                tx.commit().await.map_err(|err| {
                    error!("Failed to commit transaction: {:?}", err);
                    BusinessError::internal(format!("Failed to commit database transaction: {err}"))
                })?;
                debug!("Transaction committed successfully.");
                Ok(result)
            }
            Err(err) => {
                // `Transaction` rolls back on drop when the operation fails.
                debug!("Rolling back transaction due to error: {:?}", err);
                Err(err)
            }
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
}
