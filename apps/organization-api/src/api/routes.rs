use crate::api::handlers::{health, organizations_controller};
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{get, post, put},
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
        .route(
            "/api/v1/organizations",
            post(organizations_controller::create_organization)
                .get(organizations_controller::get_all_organizations),
        )
        .route(
            "/api/v1/organizations/{id}",
            put(organizations_controller::update_organization)
                .delete(organizations_controller::delete_organization)
                .get(organizations_controller::getbyid_organization),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::organization_config::{
        DatabaseConfig, LoggingConfig, OrganizationAppConfig, ServerConfig,
    };
    use crate::domain::organization::{Organization, OrganizationId};
    use crate::domain::organization_repository::OrganizationRepository;
    use crate::infrastructure::in_memory_organization_repository::InMemoryOrganizationRepository;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use serde_json::{Value, json};
    use shared::entities::{AuditFields, Document, DocumentType};
    use std::sync::Arc;
    use tower::ServiceExt;
    use tsid::create_tsid;

    fn create_test_state_with_repo(repository: InMemoryOrganizationRepository) -> AppState {
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
        let db = database::DatabaseConnection::connect_lazy_with_config(
            &config.database.to_connection_config(),
        );
        AppState::new_with_repository(Arc::new(config), db, repository)
    }

    fn create_test_state() -> AppState {
        create_test_state_with_repo(InMemoryOrganizationRepository::new())
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

    #[tokio::test]
    async fn test_update_organization_success() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(create_tsid());
        let id_str = org_id.to_string();
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Acme Corp", doc)
            .with_audit(AuditFields::created("autor_original"));
        repo.save(&org).await.unwrap();

        let app = create_router(create_test_state_with_repo(repo));

        let payload = json!({
            "name": "Acme Corp Atualizada",
            "updated_by": "editor_teste"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/organizations/{id_str}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json_body["id"], id_str);
        assert_eq!(json_body["name"], "Acme Corp Atualizada");
        assert_eq!(json_body["document"]["value"], "00.000.000/0001-91");
        assert_eq!(json_body["document"]["document_type"], "CNPJ");
        assert_eq!(json_body["audit"]["created_by"], "autor_original");
        assert_eq!(json_body["audit"]["updated_by"], "editor_teste");
    }

    #[tokio::test]
    async fn test_update_organization_not_found() {
        let app = create_router(create_test_state());
        let nonexistent_id = create_tsid().to_string();

        let payload = json!({
            "name": "Acme Inexistente",
            "updated_by": "editor_teste"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/organizations/{nonexistent_id}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json_body["code"], "NOT_FOUND");
        assert_eq!(json_body["status"], 404);
    }

    #[tokio::test]
    async fn test_update_organization_empty_name() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(create_tsid());
        let id_str = org_id.to_string();
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Acme Original", doc)
            .with_audit(AuditFields::created("autor"));
        repo.save(&org).await.unwrap();

        let app = create_router(create_test_state_with_repo(repo));

        let payload = json!({
            "name": "   ",
            "updated_by": "editor"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/organizations/{id_str}"))
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
    async fn test_update_organization_empty_updater() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(create_tsid());
        let id_str = org_id.to_string();
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Acme Original", doc)
            .with_audit(AuditFields::created("autor"));
        repo.save(&org).await.unwrap();

        let app = create_router(create_test_state_with_repo(repo));

        let payload = json!({
            "name": "Nome Valido",
            "updated_by": "   "
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/organizations/{id_str}"))
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
    async fn test_update_organization_invalid_path_id() {
        let app = create_router(create_test_state());

        let payload = json!({
            "name": "Nome Valido",
            "updated_by": "editor"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/organizations/invalid-id-format")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_delete_organization_success() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(create_tsid());
        let id_str = org_id.to_string();
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Acme Corp Para Deletar", doc)
            .with_audit(AuditFields::created("autor_original"));
        repo.save(&org).await.unwrap();

        let app = create_router(create_test_state_with_repo(repo.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!(
                        "/api/v1/organizations/{id_str}?update_by=admin_deleter"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Verifica se a organização foi removida do repositório
        assert!(repo.find_by_id(&org_id).is_none());
    }

    #[tokio::test]
    async fn test_delete_organization_with_updated_by_alias() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(create_tsid());
        let id_str = org_id.to_string();
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Acme Corp Para Deletar", doc)
            .with_audit(AuditFields::created("autor_original"));
        repo.save(&org).await.unwrap();

        let app = create_router(create_test_state_with_repo(repo.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!(
                        "/api/v1/organizations/{id_str}?updated_by=admin_deleter"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_organization_not_found() {
        let app = create_router(create_test_state());
        let nonexistent_id = create_tsid().to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!(
                        "/api/v1/organizations/{nonexistent_id}?update_by=admin"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json_body["code"], "NOT_FOUND");
        assert_eq!(json_body["status"], 404);
    }

    #[tokio::test]
    async fn test_delete_organization_empty_updater() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(create_tsid());
        let id_str = org_id.to_string();
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Acme Corp", doc);
        repo.save(&org).await.unwrap();

        let app = create_router(create_test_state_with_repo(repo));

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!(
                        "/api/v1/organizations/{id_str}?update_by=%20%20%20"
                    ))
                    .body(Body::empty())
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
    async fn test_get_organization_by_id_success() {
        let repo = InMemoryOrganizationRepository::new();
        let org_id = OrganizationId::new(create_tsid());
        let id_str = org_id.to_string();
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Acme Corp Para Consulta", doc)
            .with_audit(AuditFields::created("admin"));
        repo.save(&org).await.unwrap();

        let app = create_router(create_test_state_with_repo(repo));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/organizations/{id_str}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json_body["id"], id_str);
        assert_eq!(json_body["name"], "Acme Corp Para Consulta");
    }

    #[tokio::test]
    async fn test_get_all_organizations_without_params() {
        let repo = InMemoryOrganizationRepository::new();
        for i in 1..=3 {
            let org_id = OrganizationId::new(create_tsid());
            let doc = Document::new(format!("{:014}", i), DocumentType::Cnpj);
            let org = Organization::new(org_id, format!("Org {}", i), doc)
                .with_audit(AuditFields::created("admin"));
            repo.save(&org).await.unwrap();
        }

        let app = create_router(create_test_state_with_repo(repo));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/organizations")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        let arr = json_body["content"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(json_body["total_elements"], 3);
        assert_eq!(json_body["total_pages"], 1);
        assert_eq!(json_body["has_next"], false);
        assert_eq!(json_body["has_previous"], false);
    }

    #[tokio::test]
    async fn test_get_all_organizations_with_filter_and_pagination() {
        let repo = InMemoryOrganizationRepository::new();
        for i in 1..=5 {
            let org_id = OrganizationId::new(create_tsid());
            let name = if i <= 3 {
                format!("Acme Branch {}", i)
            } else {
                format!("Beta Corp {}", i)
            };
            let doc = Document::new(format!("{:014}", i), DocumentType::Cnpj);
            let org = Organization::new(org_id, name, doc).with_audit(AuditFields::created("admin"));
            repo.save(&org).await.unwrap();
        }

        let app = create_router(create_test_state_with_repo(repo));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/organizations?name=Acme&page=1&page_size=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
        let json_body: Value = serde_json::from_slice(&body).unwrap();
        let arr = json_body["content"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(json_body["total_elements"], 3);
        assert_eq!(json_body["total_pages"], 2);
        assert_eq!(json_body["has_next"], true);
        assert_eq!(json_body["has_previous"], false);
        for item in arr {
            assert!(item["name"].as_str().unwrap().contains("Acme"));
        }
    }
}
