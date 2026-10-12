use crate::config::DatabaseConnectionConfig;
use crate::error::DatabaseError;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::ops::Deref;
use tracing::info;

/// Entidade responsável pelo gerenciamento de conexões e pool do banco de dados (PostgreSQL/SQLx).
///
/// Pode ser facilmente injetada em serviços, repositórios e casos de uso via `Clone` ou `Arc`.
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pool: PgPool,
}

impl DatabaseConnection {
    /// Cria uma nova instância de `DatabaseConnection` a partir de um `PgPool` existente.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Inicializa e conecta ao banco de dados utilizando a URL de conexão informada.
    pub async fn connect(url: &str) -> Result<Self, DatabaseError> {
        info!("Conectando ao banco de dados PostgreSQL...");
        let pool = PgPoolOptions::new()
            .connect(url)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        info!("Conexão com o banco de dados estabelecida com sucesso.");
        Ok(Self { pool })
    }

    /// Inicializa e conecta ao banco de dados com base nas configurações detalhadas (`DatabaseConnectionConfig`).
    pub async fn connect_with_config(config: &DatabaseConnectionConfig) -> Result<Self, DatabaseError> {
        info!(
            "Conectando ao banco de dados PostgreSQL em {}:{}/{}...",
            config.host, config.port, config.name
        );
        let connect_options = config.to_connect_options();
        let pool = config
            .to_pool_options()
            .connect_with(connect_options)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        info!("Conexão com o banco de dados estabelecida com sucesso.");
        Ok(Self { pool })
    }

    /// Cria o pool de conexão de forma preguiçosa (lazy) a partir de uma URL.
    /// Útil para inicialização rápida ou cenários de testes unitários/integrados.
    pub fn connect_lazy(url: &str) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .connect_lazy(url)
            .map_err(DatabaseError::ConnectionError)?;
        Ok(Self { pool })
    }

    /// Cria o pool de conexão de forma preguiçosa (lazy) com base em `DatabaseConnectionConfig`.
    pub fn connect_lazy_with_config(config: &DatabaseConnectionConfig) -> Self {
        let connect_options = config.to_connect_options();
        let pool = config
            .to_pool_options()
            .connect_lazy_with(connect_options);
        Self { pool }
    }

    /// Retorna uma referência ao pool subjacente do SQLx.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Consome a entidade retornando o pool `PgPool` do SQLx.
    pub fn into_inner(self) -> PgPool {
        self.pool
    }

    /// Executa um teste de conectividade (health check / ping) no banco de dados.
    pub async fn ping(&self) -> Result<(), DatabaseError> {
        sqlx::raw_sql("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(DatabaseError::ConnectionError)
    }

    /// Verifica se o pool de conexões foi fechado.
    pub fn is_closed(&self) -> bool {
        self.pool.is_closed()
    }

    /// Fecha o pool de conexões com o banco de dados.
    pub async fn close(&self) {
        self.pool.close().await;
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
