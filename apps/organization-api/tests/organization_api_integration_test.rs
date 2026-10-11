use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use organizacao_api::api::{AppState, create_router};
use organizacao_api::config::organization_config::{DatabaseConfig, LoggingConfig, OrganizationAppConfig, ServerConfig};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

fn setup_test_app() -> axum::Router {
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
    let state = AppState::new(Arc::new(config));
    create_router(state)
}

#[tokio::test]
async fn test_route_not_found() {
    let app = setup_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/non_existent_route")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_organization_with_cpf_and_cuit() {
    let app = setup_test_app();

    // Test with CPF
    let payload_cpf = json!({
        "id": 2,
        "name": "Cliente Individual",
        "document_value": "123.456.789-09",
        "document_type": "CPF",
        "created_by": "system"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/organizations")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload_cpf).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
    let json_body: Value = serde_json::from_slice(&body).unwrap();
    assert!(json_body["id"].is_string());
    assert!(!json_body["id"].as_str().unwrap().is_empty());
    assert_eq!(json_body["document"]["document_type"], "CPF");
    assert_eq!(json_body["document"]["country"], "BR");

    // Test with CUIT Argentina
    let payload_cuit = json!({
        "id": 3,
        "name": "Empresa Argentina SA",
        "document_value": "20-12345678-6",
        "document_type": "CUIT",
        "created_by": "admin_latam"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/organizations")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&payload_cuit).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
    let json_body: Value = serde_json::from_slice(&body).unwrap();
    assert!(json_body["id"].is_string());
    assert!(!json_body["id"].as_str().unwrap().is_empty());
    assert_eq!(json_body["document"]["document_type"], "CUIT");
    assert_eq!(json_body["document"]["country"], "AR");
}

#[tokio::test]
async fn test_create_organization_invalid_json() {
    let app = setup_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/organizations")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{ invalid_json }"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
