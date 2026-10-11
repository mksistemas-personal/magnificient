use crate::api::handlers::{health, organizations_controller};
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health::health_check))
        .route("/api/v1/health", get(health::health_check))
        .route("/api/v1/organizations", post(organizations_controller::create_organization))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::organization_config::{DatabaseConfig, LoggingConfig, OrganizationAppConfig, ServerConfig};
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use serde_json::{Value, json};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn create_test_state() -> AppState {
        let config = OrganizationAppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                name: "test_db".to_string(),
                username: "admin".to_string(),
                password: "secret".to_string(),
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
            },
        };
        AppState::new(Arc::new(config))
    }

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let app = create_router(create_test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json_body["status"], "UP");
        assert_eq!(json_body["service"], "organization-api");
    }

    #[tokio::test]
    async fn test_create_organization_success() {
        let app = create_router(create_test_state());

        let payload = json!({
            "id": 1,
            "name": "Acme Corp",
            "document_value": "00.000.000/0001-91",
            "document_type": "CNPJ",
            "created_by": "api_user"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/organizations")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        assert!(json_body["id"].is_string());
        assert!(!json_body["id"].as_str().unwrap().is_empty());
        assert_eq!(json_body["name"], "Acme Corp");
        assert_eq!(json_body["document"]["value"], "00.000.000/0001-91");
        assert_eq!(json_body["document"]["document_type"], "CNPJ");
        assert_eq!(json_body["audit"]["created_by"], "api_user");
    }

    #[tokio::test]
    async fn test_create_organization_invalid_document() {
        let app = create_router(create_test_state());

        let payload = json!({
            "id": 1,
            "name": "Acme Corp",
            "document_value": "111.111.111-11",
            "document_type": "CNPJ",
            "created_by": "api_user"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/organizations")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json_body["code"], "BAD_REQUEST");
        assert_eq!(json_body["status"], 400);
    }

    #[tokio::test]
    async fn test_create_organization_empty_name() {
        let app = create_router(create_test_state());

        let payload = json!({
            "id": 1,
            "name": "   ",
            "document_value": "00.000.000/0001-91",
            "document_type": "CNPJ",
            "created_by": "api_user"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/organizations")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json_body["code"], "BAD_REQUEST");
        assert_eq!(json_body["status"], 400);
    }
}
