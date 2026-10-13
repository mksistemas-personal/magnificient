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
}

impl UnitOfWork {
    /// Cria uma nova instância de `UnitOfWork` a partir de um `PgPool`.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Cria uma nova instância de `UnitOfWork` a partir de uma `DatabaseConnection`.
    pub fn from_connection(db: &DatabaseConnection) -> Self {
        Self {
            pool: db.pool().clone(),
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
        F: FnOnce(&mut Transaction<'static, Postgres>) -> Fut,
        Fut: Future<Output = Result<T, BusinessError>>,
    {
        debug!("Iniciando transação na Unidade de Trabalho (Unit of Work)...");
        let mut tx = self.pool.begin().await.map_err(|err| {
            error!("Falha ao abrir transação no banco de dados: {:?}", err);
            BusinessError::internal(format!("Falha ao abrir transação no banco de dados: {err}"))
        })?;

        match operation(&mut tx).await {
            Ok(result) => {
                debug!("Confirmando (commit) transação da Unidade de Trabalho...");
                tx.commit().await.map_err(|err| {
                    error!("Falha ao confirmar (commit) transação: {:?}", err);
                    BusinessError::internal(format!("Falha ao confirmar transação no banco de dados: {err}"))
                })?;
                debug!("Transação confirmada com sucesso.");
                Ok(result)
            }
            Err(err) => {
                debug!("Revertendo (rollback) transação devido a erro: {:?}", err);
                if let Err(rollback_err) = tx.rollback().await {
                    error!("Erro adicional ao reverter transação: {:?}", rollback_err);
                }
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
