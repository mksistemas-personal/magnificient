use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgConnection;

/// Builds an owned migrator configured with a service-specific history table.
pub fn migrator_with_table(mut migrator: Migrator, table_name: &'static str) -> Migrator {
    migrator.dangerous_set_table_name(table_name);
    migrator
}

/// Creates a tenant schema and applies the service migrations to that schema.
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
    let configured = sqlx::query("SELECT set_config('search_path', $1, false)")
        .bind(&quoted_schema)
        .execute(&mut *connection)
        .await;
    let migration_result = match configured {
        Ok(_) => match prepare_migration_history(&mut connection, migrator).await {
            Ok(()) => migrator
                .run(&mut *connection)
                .await
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send + Sync>),
            Err(err) => Err(Box::new(err) as Box<dyn std::error::Error + Send + Sync>),
        },
        Err(err) => Err(Box::new(err) as Box<dyn std::error::Error + Send + Sync>),
    };
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

/// Creates the service-specific SQLx migration table and imports only matching
/// entries from the legacy `_sqlx_migrations` table, if present. This preserves
/// migration history during rollout while allowing APIs to share a schema.
pub async fn prepare_migration_history(
    connection: &mut PgConnection,
    migrator: &Migrator,
) -> Result<(), sqlx::Error> {
    let table_name = migrator.table_name.as_ref();
    if table_name == "_sqlx_migrations" {
        return Ok(());
    }
    if !table_name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_')
        || table_name.is_empty()
    {
        return Err(sqlx::Error::Protocol(
            "Migration table name must be a simple SQL identifier".into(),
        ));
    }

    let legacy_table_exists: bool =
        sqlx::query_scalar("SELECT to_regclass('_sqlx_migrations') IS NOT NULL")
            .fetch_one(&mut *connection)
            .await?;
    if !legacy_table_exists {
        return Ok(());
    }

    let quoted_table = format!("\"{table_name}\"");
    let create_sql = format!(
        "CREATE TABLE IF NOT EXISTS {quoted_table} (\
         version BIGINT PRIMARY KEY, description TEXT NOT NULL, \
         installed_on TIMESTAMPTZ NOT NULL DEFAULT now(), success BOOLEAN NOT NULL, \
         checksum BYTEA NOT NULL, execution_time BIGINT NOT NULL)"
    );
    sqlx::raw_sql(sqlx::AssertSqlSafe(create_sql))
        .execute(&mut *connection)
        .await?;

    let versions: Vec<i64> = migrator
        .migrations
        .iter()
        .map(|migration| migration.version)
        .collect();
    let copy_sql = format!(
        "INSERT INTO {quoted_table} (version, description, installed_on, success, checksum, execution_time) \
         SELECT version, description, installed_on, success, checksum, execution_time \
         FROM _sqlx_migrations WHERE version = ANY($1) ON CONFLICT (version) DO NOTHING"
    );
    sqlx::query(sqlx::AssertSqlSafe(copy_sql))
        .bind(versions)
        .execute(&mut *connection)
        .await?;
    Ok(())
}
