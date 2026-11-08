use sqlx::PgPool;
use sqlx::migrate::Migrator;

/// Builds an owned migrator configured with a service-specific history table.
pub fn migrator_with_table(mut migrator: Migrator, table_name: &'static str) -> Migrator {
    migrator.dangerous_set_table_name(table_name);
    migrator
}

/// Combines migrations from shared and service-specific sources into one
/// ordered SQLx migrator. Migration versions must be unique across the inputs.
pub fn combine_migrators(migrators: impl IntoIterator<Item = Migrator>) -> Migrator {
    let migrations: Vec<sqlx::migrate::Migration> = migrators
        .into_iter()
        .flat_map(|migrator| migrator.migrations.into_owned())
        .collect();
    let mut versions = std::collections::HashSet::new();
    for migration in &migrations {
        assert!(
            versions.insert(migration.version),
            "Migration version {} is duplicated across migration sources",
            migration.version
        );
    }
    Migrator::with_migrations(migrations)
}

/// Creates a tenant schema and applies the combined migrations to that schema.
/// The slug is restricted to lowercase letters, digits and hyphens; the SQL
/// identifier is generated here and is never accepted as arbitrary SQL input.
pub async fn provision_tenant_schema(
    pool: &PgPool,
    tenant_slug: &str,
    migrator: &Migrator,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let slug_valid = !tenant_slug.is_empty()
        && tenant_slug.len() <= 56
        && tenant_slug.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && tenant_slug.ends_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && tenant_slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !tenant_slug.contains("--");
    if !slug_valid {
        return Err(Box::new(sqlx::Error::Protocol(
            "Invalid tenant slug".into(),
        )));
    }

    let schema = format!("tenant_{}", tenant_slug.replace('-', "_"));
    let quoted_schema = format!("\"{schema}\"");
    let mut connection = pool.acquire().await?;
    let create_schema_sql = format!("CREATE SCHEMA IF NOT EXISTS {quoted_schema}");
    sqlx::raw_sql(sqlx::AssertSqlSafe(create_schema_sql))
        .execute(&mut *connection)
        .await?;

    // Set session search_path only while this connection is checked out, then
    // always restore it before returning the connection to the shared pool.
    sqlx::query("SELECT set_config('search_path', $1, false)")
        .bind(&quoted_schema)
        .execute(&mut *connection)
        .await?;
    let migration_result = migrator.run(&mut *connection).await;
    let reset_result = sqlx::query("RESET search_path")
        .execute(&mut *connection)
        .await;

    if let Err(reset_error) = reset_result {
        // Do not return a connection with tenant session state to the pool.
        connection.close().await?;
        return Err(Box::new(reset_error));
    }
    migration_result?;
    sqlx::query("INSERT INTO public.tenants (slug, schema_name) VALUES ($1, $2) ON CONFLICT (slug) DO NOTHING")
        .bind(tenant_slug)
        .bind(&schema)
        .execute(pool)
        .await?;
    let registered_schema: String =
        sqlx::query_scalar("SELECT schema_name FROM public.tenants WHERE slug = $1")
            .bind(tenant_slug)
            .fetch_one(pool)
            .await?;
    if registered_schema != schema {
        return Err(Box::new(sqlx::Error::Protocol(
            "Tenant slug is already registered with a different schema".into(),
        )));
    }
    Ok(schema)
}
