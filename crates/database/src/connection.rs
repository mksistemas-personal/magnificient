use crate::config::DatabaseConnectionConfig;
use crate::error::DatabaseError;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::ops::Deref;
use tracing::info;

/// Manages database connections and the PostgreSQL/SQLx connection pool.
///
/// Pode ser facilmente injetada em serviços, repositórios e casos de uso via `Clone` ou `Arc`.
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pool: PgPool,
}

impl DatabaseConnection {
    /// Creates a `DatabaseConnection` from an existing `PgPool`.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initializes a database connection using the provided connection URL.
    pub async fn connect(url: &str) -> Result<Self, DatabaseError> {
        info!("Connecting to PostgreSQL database...");
        let pool = PgPoolOptions::new()
            .connect(url)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        info!("Successfully connected to the database.");
        Ok(Self { pool })
    }

    /// Initializes a database connection using the detailed `DatabaseConnectionConfig`.
    pub async fn connect_with_config(
        config: &DatabaseConnectionConfig,
    ) -> Result<Self, DatabaseError> {
        info!(
            "Connecting to PostgreSQL database at {}:{}/{}...",
            config.host, config.port, config.name
        );
        let connect_options = config.to_connect_options();
        let pool = config
            .to_pool_options()
            .connect_with(connect_options)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        info!("Successfully connected to the database.");
        Ok(Self { pool })
    }

    /// Lazily creates a connection pool from a URL.
    /// Útil para inicialização rápida ou cenários de testes unitários/integrados.
    pub fn connect_lazy(url: &str) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .connect_lazy(url)
            .map_err(DatabaseError::ConnectionError)?;
        Ok(Self { pool })
    }

    /// Lazily creates a connection pool from `DatabaseConnectionConfig`.
    pub fn connect_lazy_with_config(config: &DatabaseConnectionConfig) -> Self {
        let connect_options = config.to_connect_options();
        let pool = config.to_pool_options().connect_lazy_with(connect_options);
        Self { pool }
    }

    /// Returns a reference to the underlying SQLx pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Consome a entidade retornando o pool `PgPool` do SQLx.
    pub fn into_inner(self) -> PgPool {
        self.pool
    }

    /// Runs a database connectivity check (health check / ping).
    pub async fn ping(&self) -> Result<(), DatabaseError> {
        sqlx::raw_sql("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(DatabaseError::ConnectionError)
    }

    /// Checks whether the connection pool has been closed.
    pub fn is_closed(&self) -> bool {
        self.pool.is_closed()
    }

    /// Closes the database connection pool.
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Retorna uma instância de `UnitOfWork` associada a esta conexão para orquestração de transações.
    pub fn unit_of_work(&self) -> crate::unit_of_work::UnitOfWork {
        crate::unit_of_work::UnitOfWork::from_connection(self)
    }
}

impl Deref for DatabaseConnection {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl AsRef<PgPool> for DatabaseConnection {
    fn as_ref(&self) -> &PgPool {
        &self.pool
    }
}

impl From<PgPool> for DatabaseConnection {
    fn from(pool: PgPool) -> Self {
        Self::new(pool)
    }
}

impl From<DatabaseConnection> for PgPool {
    fn from(conn: DatabaseConnection) -> Self {
        conn.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection_lazy_creation_and_cloning() {
        let config = DatabaseConnectionConfig::new(
            "localhost".to_string(),
            5432,
            "testdb".to_string(),
            "postgres".to_string(),
            "secret".to_string(),
        );

        let db = DatabaseConnection::connect_lazy_with_config(&config);
        let db_cloned = db.clone();

        assert!(!db.is_closed());
        assert!(!db_cloned.is_closed());
        assert_eq!(db.pool().is_closed(), false);
    }

    #[tokio::test]
    async fn test_database_connection_deref_and_as_ref() {
        let config = DatabaseConnectionConfig::default();
        let db = DatabaseConnection::connect_lazy_with_config(&config);

        let pool_ref: &PgPool = &db;
        assert!(!pool_ref.is_closed());

        let as_ref_pool: &PgPool = db.as_ref();
        assert!(!as_ref_pool.is_closed());
    }
}
