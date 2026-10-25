use database::DatabaseConnection;
use messaging::{RabbitMqPublisher, create_rabbitmq_client};
use organizacao_api::api::{AppState, create_router};
use organizacao_api::config::organization_config::OrganizationAppConfig;
use organizacao_api::infrastructure::RabbitmqSeed;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

const DEFAULT_LOG_FILTER: &str = "info,organizacao_api=debug,tower_http=debug,database=debug";

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| DEFAULT_LOG_FILTER.into()),
        )
        .init();
}

async fn setup_database(
    config: &OrganizationAppConfig,
) -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
    let db =
        DatabaseConnection::connect_with_config(&config.database.to_connection_config()).await?;
    info!("Executing database migrations...");
    sqlx::migrate!("./migrations").run(db.pool()).await?;
    info!("Migrations run successfully.");
    Ok(db)
}

async fn setup_messaging(
    config: &OrganizationAppConfig,
) -> Result<Arc<RabbitMqPublisher>, Box<dyn std::error::Error>> {
    info!("Initializing RabbitMQ publisher...");
    let rabbitmq_client = create_rabbitmq_client(&config.rabbitmq).await?;
    let rabbitmq_seed = RabbitmqSeed::new(&rabbitmq_client);
    rabbitmq_seed.create().await?;
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
