use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Failed to connect to the database: {0}")]
    ConnectionError(#[from] sqlx::Error),

    #[error("Invalid database configuration: {0}")]
    ConfigurationError(String),

    #[error("Migration failed: {0}")]
    MigrationError(String),
}
