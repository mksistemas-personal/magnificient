use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use cliente_api::adapters::api::{
    AppState, ClientResponseDto, CreateClientDto, UpdateClientDto, create_router,
};
use cliente_api::application::{
    CreateClientRequest, DeleteClientRequest, FindClientsRequest, GetClientByIdRequest,
};
use cliente_api::config::{ClientAppConfig, LoggingConfig, ServerConfig};
use cliente_api::domain::ClientId;
use cliente_api::infrastructure::InMemoryClientRepository;
use database::DatabaseConnectionConfig;
use messaging::RabbitMqConfig;
use serde_json::{Value, json};
use shared::entities::{Address, InMemoryAddressRepository};
use shared::errors::api_error::ApiError;
use shared::types::PageRequest;
use std::sync::Arc;
use tower::ServiceExt;
use tsid::TSID;

fn setup_test_state() -> AppState {
    let config = ClientAppConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        database: DatabaseConnectionConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "test_db".to_string(),
            username: "admin".to_string(),
            password: "secret".to_string(),
            max_connections: None,
            min_connections: None,
            acquire_timeout_secs: None,
            max_lifetime_secs: None,
            idle_timeout_secs: None,
        },
        rabbitmq: RabbitMqConfig {
            rabbitmq_url: "amqp://rabbitmq:rabbitmq@localhost:5672/%2f".to_string(),
            rabbitmq_prefetch_count: 10,
            rabbitmq_reconnect_max_attempts: 0,
            rabbitmq_reconnect_initial_ms: 1000,
            rabbitmq_reconnect_max_ms: 30000,
            rabbitmq_retry_max_attempts: 3,
            rabbitmq_retry_initial_ms: 1000,
            rabbitmq_retry_max_ms: 10000,
        },
        logging: LoggingConfig {
            level: "debug".to_string(),
        },
    };
    let db = database::DatabaseConnection::connect_lazy_with_config(&config.database);
    let repository = InMemoryClientRepository::new();
    let address_repository = InMemoryAddressRepository::new();
    AppState::new_with_repositories(Arc::new(config), db, repository, address_repository)
}

fn setup_test_app() -> axum::Router {
    let state = setup_test_state();
    create_router(state)
}

fn map_api_error<E>(error: E) -> ApiError
where
    E: Into<ApiError>,
{
    error.into()
}

/// Simula o fluxo completo do handler create_client: DTO parsing → request → use case → response DTO
async fn simulate_create_client(
    state: &AppState,
    dto: CreateClientDto,
) -> Result<(StatusCode, ClientResponseDto), ApiError> {
    let request: CreateClientRequest = dto.try_into()?;
    let client = state
        .create_client
        .execute(request)
        .await
        .map_err(map_api_error)?;
    Ok((StatusCode::CREATED, ClientResponseDto::from(client)))
}

/// Simula o fluxo completo do handler update_client
async fn simulate_update_client(
    state: &AppState,
    client_id: ClientId,
    dto: UpdateClientDto,
) -> Result<(StatusCode, ClientResponseDto), ApiError> {
    let request = dto.into_request(client_id)?;
    let client = state
        .update_client
        .execute(request)
        .await
        .map_err(map_api_error)?;
    Ok((StatusCode::OK, ClientResponseDto::from(client)))
}

/// Simula o fluxo completo do handler delete_client
async fn simulate_delete_client(
    state: &AppState,
    client_id: ClientId,
    updated_by: String,
) -> Result<StatusCode, ApiError> {
    state
        .delete_client
        .execute(DeleteClientRequest {
            id: client_id,
            updated_by,
        })
        .await
        .map_err(map_api_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Simula o fluxo completo do handler get_client
async fn simulate_get_client(
    state: &AppState,
    client_id: ClientId,
) -> Result<(StatusCode, ClientResponseDto), ApiError> {
    let client = state
        .get_client
        .execute(GetClientByIdRequest { id: client_id })
        .await
        .map_err(map_api_error)?;
    Ok((StatusCode::OK, ClientResponseDto::from(client)))
}

/// Simula o fluxo completo do handler find_clients
async fn simulate_find_clients(
    state: &AppState,
    name: Option<String>,
    organization_id: Option<cliente_api::domain::OrganizationId>,
    page: u32,
    page_size: u32,
) -> Result<(StatusCode, Value), ApiError> {
    let page_result = state
        .find_clients
        .execute(FindClientsRequest {
            name,
            organization_id,
            page_request: PageRequest::new(page, page_size),
        })
        .await
        .map_err(map_api_error)?
        .map(ClientResponseDto::from);
    Ok((StatusCode::OK, serde_json::to_value(page_result).unwrap()))
}

#[tokio::test]
async fn test_health_check() {
    let app = setup_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
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
    assert_eq!(json_body["service"], "client-api");
}

#[tokio::test]
async fn test_health_check_v1() {
    let app = setup_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 64).await.unwrap();
    let json_body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json_body["status"], "UP");
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
async fn test_create_client_invalid_json() {
    let app = setup_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenant1/clients")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{ invalid_json }"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_client_with_address_flow() {
    let state = setup_test_state();
    let org_id = cliente_api::domain::OrganizationId::new(TSID::from(1000u64));

    // 1. Criar cliente com endereço
    let dto = CreateClientDto {
        organization_id: org_id.to_string(),
        name: "Empresa Original".to_string(),
        nickname: Some("Emp".to_string()),
        second_address: Some(Address::new(
            "Av. Paulista",
            "1000",
            "Sala 101",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "01310-100",
            "Brasil",
            "BR",
            "Perto do metro",
        )),
        created_by: "criador".to_string(),
    };

    let (status, created) = simulate_create_client(&state, dto).await.unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created.name, "Empresa Original");
    assert_eq!(created.nickname, Some("Emp".to_string()));
    assert!(created.second_address.is_some());
    assert_eq!(
        created.second_address.as_ref().unwrap().street,
        "Av. Paulista"
    );
    assert!(created.audit.is_some());
    let client_id = ClientId::new(TSID::try_from(created.id.as_str()).unwrap());

    // 2. Atualizar cliente
    let update_dto = UpdateClientDto {
        organization_id: org_id.to_string(),
        name: "Empresa Atualizada".to_string(),
        nickname: None,
        second_address: None,
        updated_by: "editor".to_string(),
    };

    let (status, updated) = simulate_update_client(&state, client_id, update_dto)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated.name, "Empresa Atualizada");
    assert_eq!(updated.audit.as_ref().unwrap().updated_by(), "editor");

    // 3. Buscar cliente por ID
    let (status, got) = simulate_get_client(&state, client_id).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(got.id, created.id);
    assert_eq!(got.name, "Empresa Atualizada");

    // 4. Deletar cliente
    let status = simulate_delete_client(&state, client_id, "deleter".to_string())
        .await
        .unwrap();
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 5. Tentar buscar cliente deletado deve retornar erro
    let result = simulate_get_client(&state, client_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_and_find_clients_flow() {
    let state = setup_test_state();
    let org_id = cliente_api::domain::OrganizationId::new(TSID::from(2000u64));

    // 1. Criar vários clientes
    let mut client_ids = Vec::new();
    for i in 1..=3 {
        let dto = CreateClientDto {
            organization_id: org_id.to_string(),
            name: format!("Cliente Teste {}", i),
            nickname: Some(format!("CT{}", i)),
            second_address: None,
            created_by: "sistema".to_string(),
        };

        let (status, created) = simulate_create_client(&state, dto).await.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        client_ids.push(ClientId::new(TSID::try_from(created.id.as_str()).unwrap()));
    }

    // 2. Listar todos sem filtro
    let (status, page) = simulate_find_clients(&state, None, None, 1, 20)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);
    let list = page["content"].as_array().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(page["total_elements"], 3);
    assert_eq!(page["total_pages"], 1);

    // 3. Listar com filtro de nome
    let (status, page) =
        simulate_find_clients(&state, Some("Cliente Teste 1".to_string()), None, 1, 20)
            .await
            .unwrap();
    assert_eq!(status, StatusCode::OK);
    let list = page["content"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "Cliente Teste 1");
    assert_eq!(page["total_elements"], 1);

    // 4. Listar com filtro de organization_id e paginação
    let (status, page) = simulate_find_clients(&state, None, Some(org_id), 1, 2)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);
    let list = page["content"].as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(page["total_elements"], 3);
    assert_eq!(page["total_pages"], 2);
}

#[tokio::test]
async fn test_delete_client_not_found() {
    let state = setup_test_state();
    let non_existent_id = ClientId::new(TSID::from(3000u64));

    let result = simulate_delete_client(&state, non_existent_id, "deleter".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_client_not_found() {
    let state = setup_test_state();
    let non_existent_id = ClientId::new(TSID::from(4000u64));

    let result = simulate_get_client(&state, non_existent_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_client_not_found() {
    let state = setup_test_state();
    let non_existent_id = ClientId::new(TSID::from(5000u64));
    let org_id = cliente_api::domain::OrganizationId::new(TSID::from(5001u64));

    let update_dto = UpdateClientDto {
        organization_id: org_id.to_string(),
        name: "Nome Atualizado".to_string(),
        nickname: None,
        second_address: None,
        updated_by: "editor".to_string(),
    };

    let result = simulate_update_client(&state, non_existent_id, update_dto).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_find_clients_empty_list() {
    let state = setup_test_state();

    let (status, page) = simulate_find_clients(&state, None, None, 1, 20)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::OK);
    let list = page["content"].as_array().unwrap();
    assert_eq!(list.len(), 0);
    assert_eq!(page["total_elements"], 0);
    assert_eq!(page["total_pages"], 0);
}

#[tokio::test]
async fn test_create_client_dto_parsing() {
    let org_id = TSID::from(6000u64).to_string();

    // Testar parsing de JSON para CreateClientDto via router (sem tenant, só valida DTO)
    let payload = json!({
        "organization_id": org_id,
        "name": "Teste DTO",
        "nickname": "DTO",
        "created_by": "test"
    });

    // Verificar que o DTO é serializado/desserializado corretamente
    let dto: CreateClientDto = serde_json::from_value(payload).unwrap();
    assert_eq!(dto.name, "Teste DTO");
    assert_eq!(dto.nickname, Some("DTO".to_string()));
    assert_eq!(dto.created_by, "test");

    // Executar criação via simulate
    let state = setup_test_state();
    let (status, created) = simulate_create_client(&state, dto).await.unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created.name, "Teste DTO");
}

#[tokio::test]
async fn test_create_client_normalizes_nickname() {
    let state = setup_test_state();
    let org_id = cliente_api::domain::OrganizationId::new(TSID::from(7000u64));

    let dto = CreateClientDto {
        organization_id: org_id.to_string(),
        name: "Cliente com Nickname".to_string(),
        nickname: Some("  Nickname com espacos  ".to_string()),
        second_address: None,
        created_by: "test".to_string(),
    };

    let (_, created) = simulate_create_client(&state, dto).await.unwrap();
    // O serviço deve normalizar o nickname (trim)
    assert_eq!(created.nickname, Some("Nickname com espacos".to_string()));
}
