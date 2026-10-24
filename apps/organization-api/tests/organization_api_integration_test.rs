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
            max_connections: None,
            min_connections: None,
            acquire_timeout_secs: None,
        },
        logging: LoggingConfig {
            level: "debug".to_string(),
        },
    };
    let db = database::DatabaseConnection::connect_lazy_with_config(&config.database.to_connection_config());
    let repository = organizacao_api::infrastructure::InMemoryOrganizationRepository::new();
    let state = AppState::new_with_repository(Arc::new(config), db, repository);
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

#[tokio::test]
async fn test_create_and_delete_organization_flow() {
    let app = setup_test_app();

    // 1. Criar organização
    let payload = json!({
        "name": "Org Para Deletar",
        "document_value": "00.000.000/0001-91",
        "document_type": "CNPJ",
        "created_by": "criador"
    });

    let res_create = app
        .clone()
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

    assert_eq!(res_create.status(), StatusCode::CREATED);
    let body = to_bytes(res_create.into_body(), 1024 * 64).await.unwrap();
    let json_body: Value = serde_json::from_slice(&body).unwrap();
    let org_id = json_body["id"].as_str().unwrap();

    // 2. Deletar organização via DELETE /api/v1/organizations/{id}?update_by=deleter
    let res_delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/organizations/{org_id}?update_by=deleter"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_delete.status(), StatusCode::NO_CONTENT);

    // 3. Tentar deletar novamente deve retornar 404 (já deletada)
    let res_delete_again = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/organizations/{org_id}?update_by=deleter"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_delete_again.status(), StatusCode::NOT_FOUND);
}
