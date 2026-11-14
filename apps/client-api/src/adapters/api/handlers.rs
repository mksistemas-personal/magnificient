use crate::adapters::api::client_dto::{
    ClientResponseDto, CreateClientDto, DeleteClientParams, FindClientsParams, UpdateClientDto,
    map_api_error, parse_id,
};
use crate::adapters::api::state::AppState;
use crate::application::{
    CreateClientRequest, DeleteClientRequest, FindClientsRequest, GetClientByIdRequest,
};
use crate::domain::ClientId;
use crate::infrastructure::tenant_state;
use axum::extract::{Path, Query, State};
use axum::{Json, http::StatusCode, response::IntoResponse};
use client_contracts::v1::{ClientAddress, ClientEvent, EVENT_NAME, EVENT_VERSION};
use messaging::{MessageHeaders, MessagePublisher};
use shared::entities::Address;
use shared::errors::api_error::ApiError;
use shared::types::PageRequest;
use tsid::TSID;

const CLIENT_EXCHANGE_NAME: &str = "client.output";
const ROUTING_KEY: &str = "*";
const EVENT_SOURCE: &str = "client-api";

pub async fn create_client(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
    Json(payload): Json<CreateClientDto>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let request: CreateClientRequest = payload.try_into()?;
    let client = state
        .create_client
        .execute(request)
        .await
        .map_err(map_api_error)?;
    let event = client_added_event(&client);
    publish_message(state, &event).await?;
    Ok((StatusCode::CREATED, Json(ClientResponseDto::from(client))))
}

pub async fn update_client(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
    Json(payload): Json<UpdateClientDto>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let request = payload.into_request(ClientId::new(id))?;
    let client = state
        .update_client
        .execute(request)
        .await
        .map_err(map_api_error)?;
    let event = client_updated_event(&client);
    publish_message(state, &event).await?;
    Ok((StatusCode::OK, Json(ClientResponseDto::from(client))))
}

pub async fn delete_client(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
    Query(params): Query<DeleteClientParams>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let client_id = ClientId::new(id);
    state
        .delete_client
        .execute(DeleteClientRequest {
            id: client_id,
            updated_by: params.updated_by,
        })
        .await
        .map_err(map_api_error)?;
    let event = ClientEvent::delete(client_id.to_string());
    publish_message(state, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn publish_message(state: AppState, event: &ClientEvent) -> Result<(), ApiError> {
    if let Some(publisher) = state.message_publisher.as_ref() {
        publisher
            .publish_json(
                CLIENT_EXCHANGE_NAME,
                ROUTING_KEY,
                event,
                Some(MessageHeaders::create_default(
                    EVENT_NAME,
                    EVENT_SOURCE,
                    EVENT_VERSION,
                    state.tenant.as_str(),
                )),
            )
            .await?;
    }
    Ok(())
}

fn client_added_event(client: &crate::domain::Client) -> ClientEvent {
    ClientEvent::add(
        client.id().to_string(),
        client.organization_id().to_string(),
        client.name().to_string(),
        client.nickname().map(String::from),
        client.second_address().map(client_address_event),
    )
}

fn client_updated_event(client: &crate::domain::Client) -> ClientEvent {
    ClientEvent::update(
        client.id().to_string(),
        client.organization_id().to_string(),
        client.name().to_string(),
        client.nickname().map(String::from),
        client.second_address().map(client_address_event),
    )
}

fn client_address_event(address: &Address) -> ClientAddress {
    ClientAddress {
        street: address.street.clone(),
        number: address.number.clone(),
        complement: address.complement.clone(),
        neighborhood: address.neighborhood.clone(),
        city: address.city.clone(),
        state: address.state.clone(),
        state_code: address.state_code.clone(),
        postal_code: address.postal_code.clone(),
        country: address.country.clone(),
        country_code: address.country_code.clone(),
        reference: address.reference.clone(),
    }
}

pub async fn get_client(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let client = state
        .get_client
        .execute(GetClientByIdRequest {
            id: ClientId::new(id),
        })
        .await
        .map_err(map_api_error)?;
    Ok((StatusCode::OK, Json(ClientResponseDto::from(client))))
}

pub async fn find_clients(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
    Query(params): Query<FindClientsParams>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let organization_id = params
        .organization_id
        .as_deref()
        .map(parse_id)
        .transpose()?;
    let page = state
        .find_clients
        .execute(FindClientsRequest {
            name: params.name,
            organization_id,
            page_request: PageRequest::new(
                params.page.unwrap_or(1),
                params.page_size.unwrap_or(20),
            ),
        })
        .await
        .map_err(map_api_error)?
        .map(ClientResponseDto::from);
    Ok((StatusCode::OK, Json(page)))
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "UP",
            "service": "client-api",
            "server": {
                "host": state.config.server.host,
                "port": state.config.server.port
            }
        })),
    )
}
