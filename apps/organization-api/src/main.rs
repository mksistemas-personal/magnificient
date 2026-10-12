use database::DatabaseConnection;
use organizacao_api::api::{AppState, create_router};
use organizacao_api::config::organization_config::OrganizationAppConfig;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,organizacao_api=debug,tower_http=debug,database=debug".into()),
        )
        .init();

    let config = OrganizationAppConfig::load().expect("Falha ao carregar configurações da aplicação");
    let shared_config = Arc::new(config);

    // Inicializa a conexão com o banco de dados PostgreSQL via SQLx
    let db = DatabaseConnection::connect_with_config(&shared_config.database.to_connection_config()).await?;

    let server_addr = format!("{}:{}", shared_config.server.host, shared_config.server.port);

    let state = AppState::new(Arc::clone(&shared_config), db);
    let app = create_router(state);

    let listener = TcpListener::bind(&server_addr).await?;
    info!("Organizacao API inicializada e escutando em http://{}", server_addr);

    axum::serve(listener, app).await?;
    Ok(())
}
