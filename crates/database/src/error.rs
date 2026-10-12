use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Erro ao conectar ao banco de dados: {0}")]
    ConnectionError(#[from] sqlx::Error),

    #[error("Configuração de banco de dados inválida: {0}")]
    ConfigurationError(String),

    #[error("Erro na execução de migração: {0}")]
    MigrationError(String),
}
