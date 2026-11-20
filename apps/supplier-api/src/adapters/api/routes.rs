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
            "/api/v1/{tenant}/suppliers",
            post(handlers::create_supplier).get(handlers::find_suppliers),
        )
        .route(
            "/api/v1/{tenant}/suppliers/{id}",
            get(handlers::get_supplier)
                .put(handlers::update_supplier)
                .delete(handlers::delete_supplier),
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
