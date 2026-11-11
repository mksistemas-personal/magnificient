use cliente_api::api::{AppState, create_router};
use cliente_api::config::ClientAppConfig;
use cliente_api::consumer::OrganizationEventHandler;
use cliente_api::infrastructure::RabbitmqSeed;
use database::{
    DatabaseConnection, combine_migrators, migrator_with_table, provision_tenant_schema,
};
use messaging::{MessageConsumerRegistry, RabbitMqPublisher, create_rabbitmq_client};
use organization_contracts::v1::{EVENT_NAME, EVENT_VERSION, OrganizationEvent};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

fn init_tracing(config: &ClientAppConfig) {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.logging.level.clone().into()),
        )
        .init();
}

async fn setup_database(
    config: &ClientAppConfig,
) -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
    let db = DatabaseConnection::connect_with_config(&config.database).await?;
    let migrator = migrator_with_table(
        combine_migrators([shared::database::migrator(), sqlx::migrate!("./migrations")]),
        "client_api_migrations",
    );
    let tenant_slugs: Vec<String> =
        sqlx::query_scalar("SELECT slug FROM public.tenants WHERE active = TRUE ORDER BY slug")
            .fetch_all(db.pool())
            .await?;

    for tenant in tenant_slugs {
        info!("Applying client schema migrations for tenant '{tenant}'...");
        provision_tenant_schema(db.pool(), &tenant, &migrator)
            .await
            .map_err(|error| std::io::Error::other(error.to_string()))?;
    }
    Ok(db)
}

async fn setup_messaging(
    config: &ClientAppConfig,
) -> Result<(Arc<RabbitMqPublisher>, MessageConsumerRegistry), Box<dyn std::error::Error>> {
    info!("Initializing RabbitMQ publisher...");
    let publisher_client = create_rabbitmq_client(&config.rabbitmq).await?;
    RabbitmqSeed::new(&publisher_client).create().await?;
    let publisher = Arc::new(RabbitMqPublisher::new(publisher_client));
    let consumer_client = create_rabbitmq_client(&config.rabbitmq).await?;
    let mut registry = MessageConsumerRegistry::new(consumer_client);
    registry.register::<OrganizationEvent, _>(
        EVENT_NAME,
        EVENT_VERSION,
        OrganizationEventHandler,
    )?;
    registry.start("client-input.main");
    Ok((publisher, registry))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(ClientAppConfig::load()?);
    init_tracing(&config);
    let db = setup_database(&config).await?;
    let address = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&address).await?;
    let (message_publisher, consumer_registry) = setup_messaging(&config).await?;
    let state = AppState::new_with_publisher(config.clone(), db, message_publisher);

    info!("Client API listening on http://{address}");
    let server_result = axum::serve(listener, create_router(state)).await;
    consumer_registry.shutdown().await;
    server_result?;
    Ok(())
}
