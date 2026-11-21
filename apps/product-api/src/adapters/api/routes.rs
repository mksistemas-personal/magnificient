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
            "/api/v1/{tenant}/products",
            post(handlers::create_product).get(handlers::find_products),
        )
        .route(
            "/api/v1/{tenant}/products/{id}",
            get(handlers::get_product)
                .put(handlers::update_product)
                .delete(handlers::delete_product),
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
