use cliente_api::api::{AppState, create_router};
use cliente_api::config::ClientAppConfig;
use database::{DatabaseConnection, provision_tenant_schema};
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
    let migrator = sqlx::migrate!("./migrations");
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(ClientAppConfig::load()?);
    init_tracing(&config);
    let db = setup_database(&config).await?;
    let state = AppState::new(config.clone(), db);
    let address = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&address).await?;
    info!("Client API listening on http://{address}");
    axum::serve(listener, create_router(state)).await?;
    Ok(())
}
