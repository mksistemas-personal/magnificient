use crate::api::state::AppState;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "UP",
            "service": "organization-api",
            "server": {
                "host": state.config.server.host,
                "port": state.config.server.port
            }
        })),
    )
}
