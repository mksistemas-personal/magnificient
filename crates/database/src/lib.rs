pub mod config;
pub mod connection;
pub mod error;

pub use config::DatabaseConnectionConfig;
pub use connection::DatabaseConnection;
pub use error::DatabaseError;

// Re-export sqlx core types for convenience
pub use sqlx::{self, PgPool, Pool, Postgres};
