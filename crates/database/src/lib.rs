pub mod config;
pub mod connection;
pub mod error;
pub mod executor;
mod tenant_provisioning;
pub mod unit_of_work;

pub use config::DatabaseConnectionConfig;
pub use connection::DatabaseConnection;
pub use error::DatabaseError;
pub use executor::ConnectionExecutor;
pub use tenant_provisioning::{combine_migrators, migrator_with_table, provision_tenant_schema};
pub use unit_of_work::UnitOfWork;

// Re-export sqlx core types for convenience
pub use sqlx::{self, PgPool, Pool, Postgres, Transaction};
