use crate::adapters::api::handlers;
use crate::adapters::api::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/v1/health", get(handlers::health_check))
        .route(
            "/api/v1/{tenant}/clients",
            post(handlers::create_client).get(handlers::find_clients),
        )
        .route(
            "/api/v1/{tenant}/clients/{id}",
            get(handlers::get_client)
                .put(handlers::update_client)
                .delete(handlers::delete_client),
        )
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
