use database::{
    DatabaseConnection, combine_migrators, migrator_with_table, provision_tenant_schema,
};
use messaging::{RabbitMqPublisher, create_rabbitmq_client};
use organizacao_api::api::{AppState, create_router};
use organizacao_api::config::organization_config::OrganizationAppConfig;
use organizacao_api::infrastructure::RabbitmqSeed;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

//const DEFAULT_LOG_FILTER: &str = "info,organizacao_api=debug,tower_http=debug,database=debug";
const DEBUG_LOG_FILTER: &str = "debug,organizacao_api=debug,tower_http=debug,database=debug";

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| DEBUG_LOG_FILTER.into()),
        )
        .init();
}

async fn setup_database(
    config: &OrganizationAppConfig,
) -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
    let db =
        DatabaseConnection::connect_with_config(&config.database.to_connection_config()).await?;
    info!("Executing database migrations...");
    let mut migrator = migrator_with_table(
        combine_migrators([shared::database::migrator(), sqlx::migrate!("./migrations")]),
        "organization_api_migrations",
    );
    // The organization-specific migrations were consolidated into one script.
    // Ignore their retired version rows in databases that have not yet been reset.
    migrator.set_ignore_missing(true);
    migrator.run(db.pool()).await?;
    info!("Control schema migrations run successfully.");

    let tenant_slugs: Vec<String> =
        sqlx::query_scalar("SELECT slug FROM public.tenants ORDER BY slug")
            .fetch_all(db.pool())
            .await?;

    for tenant_slug in tenant_slugs {
        info!("Executing migrations for tenant '{}'...", tenant_slug);
        provision_tenant_schema(db.pool(), &tenant_slug, &migrator)
            .await
            .map_err(|err| err as Box<dyn std::error::Error>)?;
    }

    info!("Tenant schema migrations run successfully.");
    Ok(db)
}

async fn setup_messaging(
    config: &OrganizationAppConfig,
) -> Result<Arc<RabbitMqPublisher>, Box<dyn std::error::Error>> {
    info!("Initializing RabbitMQ publisher...");
    let rabbitmq_client = create_rabbitmq_client(&config.rabbitmq).await?;
    RabbitmqSeed::new(&rabbitmq_client).create().await?;
    Ok(Arc::new(RabbitMqPublisher::new(rabbitmq_client)))
}

async fn run_server(
    config: Arc<OrganizationAppConfig>,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = format!("{}:{}", config.server.host, config.server.port);
    let app = create_router(state);
    let listener = TcpListener::bind(&server_addr).await?;
    info!("Organization API listening in http://{}", server_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = Arc::new(OrganizationAppConfig::load().expect("Fail to load configurations"));
    let db = setup_database(&config).await?;
    let message_publisher = setup_messaging(&config).await?;

    let state = AppState::new_with_publisher(Arc::clone(&config), db, message_publisher);

    run_server(config, state).await?;

    Ok(())
}
