use cliente_api::adapters::api::AppState;
use cliente_api::adapters::consumer::OrganizationEventHandler;
use cliente_api::config::ClientAppConfig;
use database::{
    DatabaseConnection, DatabaseConnectionConfig, combine_migrators, migrator_with_table,
    provision_tenant_schema,
};
use messaging::{MessageHandler, MessageMetadata};
use organization_contracts::v1::OrganizationEvent;
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tsid::TSID;

struct TestTenant {
    db: DatabaseConnection,
    slug: String,
    schema: String,
}

impl TestTenant {
    async fn create(
        database_config: &DatabaseConnectionConfig,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let db = DatabaseConnection::connect_with_config(database_config).await?;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let slug = format!("client-api-{}-{timestamp}-it", std::process::id());
        let schema = format!("tenant_{}", slug.replace('-', "_"));
        let create_schema = format!("CREATE SCHEMA \"{schema}\"");
        sqlx::raw_sql(sqlx::AssertSqlSafe(create_schema))
            .execute(db.pool())
            .await?;

        let organization_migrations = migrator_with_table(
            combine_migrators([
                shared::database::migrator(),
                sqlx::migrate!("../organization-api/migrations"),
            ]),
            "organization_api_it_migrations",
        );
        run_migrations_in_schema(db.pool(), &schema, &organization_migrations).await?;

        let client_migrations = migrator_with_table(
            combine_migrators([shared::database::migrator(), sqlx::migrate!("./migrations")]),
            "client_api_it_migrations",
        );
        let provisioned_schema =
            provision_tenant_schema(db.pool(), &slug, &client_migrations).await?;
        assert_eq!(provisioned_schema, schema);

        Ok(Self { db, slug, schema })
    }

    async fn cleanup(&self) -> Result<(), sqlx::Error> {
        let drop_schema = format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", self.schema);
        sqlx::raw_sql(sqlx::AssertSqlSafe(drop_schema))
            .execute(self.db.pool())
            .await?;
        sqlx::query("DELETE FROM public.tenants WHERE slug = $1")
            .bind(&self.slug)
            .execute(self.db.pool())
            .await?;
        Ok(())
    }
}

async fn run_migrations_in_schema(
    pool: &PgPool,
    schema: &str,
    migrator: &sqlx::migrate::Migrator,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut connection = pool.acquire().await?;
    sqlx::query("SELECT set_config('search_path', $1, false)")
        .bind(format!("\"{schema}\""))
        .execute(&mut *connection)
        .await?;
    let migration_result = migrator.run(&mut *connection).await;
    let reset_result = sqlx::query("RESET search_path")
        .execute(&mut *connection)
        .await;
    if let Err(error) = reset_result {
        connection.close().await?;
        return Err(Box::new(error));
    }
    migration_result?;
    Ok(())
}

fn test_app_state(config: ClientAppConfig, db: DatabaseConnection) -> AppState {
    AppState::new(Arc::new(config), db)
}

#[tokio::test]
async fn delete_organization_event_only_deletes_clients_in_its_tenant_and_organization() {
    let config = ClientAppConfig::load().expect("failed to load client-api configuration");
    let fixture = TestTenant::create(&config.database)
        .await
        .expect("failed to prepare PostgreSQL integration schema");

    let organization_id = TSID::from(8_765_432u64);
    let other_organization_id = TSID::from(8_765_433u64);
    let organization_id_db = organization_id.number() as i64;
    let other_organization_id_db = other_organization_id.number() as i64;
    let organization_id_string = organization_id.to_string();

    let insert_organization = format!(
        "INSERT INTO \"{}\".organizations \
         (id, name, document_value, document_type, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6)",
        fixture.schema
    );
    let seed_result = async {
        sqlx::query(sqlx::AssertSqlSafe(insert_organization.as_str()))
            .bind(organization_id_db)
            .bind("Integration test organization")
            .bind("integration-test-document")
            .bind("test")
            .bind("integration-test")
            .bind("integration-test")
            .execute(fixture.db.pool())
            .await?;
        sqlx::query(sqlx::AssertSqlSafe(insert_organization.as_str()))
            .bind(other_organization_id_db)
            .bind("Unrelated integration test organization")
            .bind("unrelated-integration-test-document")
            .bind("test")
            .bind("integration-test")
            .bind("integration-test")
            .execute(fixture.db.pool())
            .await?;

        let insert_client = format!(
            "INSERT INTO \"{}\".clients \
             (id, organization_id, name, created_by, updated_by) \
             VALUES ($1, $2, $3, $4, $5)",
            fixture.schema
        );
        for index in 0..25u64 {
            sqlx::query(sqlx::AssertSqlSafe(insert_client.as_str()))
                .bind(TSID::from(9_000_000 + index).number() as i64)
                .bind(organization_id_db)
                .bind(format!("Target client {index}"))
                .bind("integration-test")
                .bind("integration-test")
                .execute(fixture.db.pool())
                .await?;
        }
        sqlx::query(sqlx::AssertSqlSafe(insert_client.as_str()))
            .bind(TSID::from(9_000_100u64).number() as i64)
            .bind(other_organization_id_db)
            .bind("Unrelated client")
            .bind("integration-test")
            .bind("integration-test")
            .execute(fixture.db.pool())
            .await?;
        Ok::<(), sqlx::Error>(())
    }
    .await;

    let handler_result = if seed_result.is_ok() {
        let handler =
            OrganizationEventHandler::new(test_app_state(config.clone(), fixture.db.clone()));
        handler
            .handle(
                OrganizationEvent::delete(organization_id_string),
                MessageMetadata {
                    event_type: "OrganizationEvent".to_owned(),
                    version: 100,
                    message_id: Some("organization-delete-it".to_owned()),
                    source: Some("organization-api".to_owned()),
                    tenant: Some(fixture.slug.clone()),
                },
            )
            .await
    } else {
        Ok(())
    };

    let target_rows_query = format!(
        "SELECT deleted, updated_by FROM \"{}\".clients \
             WHERE organization_id = $1 ORDER BY id",
        fixture.schema
    );
    let target_rows =
        sqlx::query_as::<_, (bool, String)>(sqlx::AssertSqlSafe(target_rows_query.as_str()))
            .bind(organization_id_db)
            .fetch_all(fixture.db.pool())
            .await;
    let unrelated_rows_query = format!(
        "SELECT deleted FROM \"{}\".clients WHERE organization_id = $1",
        fixture.schema
    );
    let unrelated_rows =
        sqlx::query_scalar::<_, bool>(sqlx::AssertSqlSafe(unrelated_rows_query.as_str()))
            .bind(other_organization_id_db)
            .fetch_all(fixture.db.pool())
            .await;

    fixture
        .cleanup()
        .await
        .expect("failed to clean up PostgreSQL integration schema");

    seed_result.expect("failed to seed PostgreSQL integration schema");
    assert!(
        handler_result.is_ok(),
        "handler failed: {}",
        handler_result.unwrap_err().message
    );
    let target_rows = target_rows.expect("failed to query target clients");
    let unrelated_rows = unrelated_rows.expect("failed to query unrelated clients");

    assert_eq!(target_rows.len(), 25);
    assert!(
        target_rows
            .iter()
            .all(|(deleted, updated_by)| *deleted && updated_by == "organization-api")
    );
    assert_eq!(unrelated_rows, vec![false]);
}
