use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use crate::api::state::AppState;

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
