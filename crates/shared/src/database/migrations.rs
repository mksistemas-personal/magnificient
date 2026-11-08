use sqlx::migrate::Migrator;

/// Returns the database migrations owned by the shared crate.
pub fn migrator() -> Migrator {
    Migrator::with_migrations(sqlx::migrate!("./migrations").migrations.to_vec())
}
