pub mod config;
pub mod connection;
pub mod error;
pub mod unit_of_work;

pub use config::DatabaseConnectionConfig;
pub use connection::DatabaseConnection;
pub use error::DatabaseError;
pub use unit_of_work::UnitOfWork;

// Re-export sqlx core types for convenience
pub use sqlx::{self, PgPool, Pool, Postgres, Transaction};
