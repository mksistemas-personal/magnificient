use crate::adapters::api::product_dto::{
    ProductResponseDto, CreateProductDto, DeleteProductParams, FindProductsParams, UpdateProductDto,
    map_api_error, parse_id,
};
use crate::adapters::api::state::AppState;
use crate::application::{
    CreateProductRequest, DeleteProductRequest, FindProductsRequest, GetProductByIdRequest,
};
use crate::constants::{PRODUCT_OUTPUT_EXCHANGE, ROUTING_KEY, EVENT_SOURCE};
use crate::domain::ProductId;
use crate::infrastructure::tenant_state;
use axum::extract::{Path, Query, State};
use axum::{Json, http::StatusCode, response::IntoResponse};
use messaging::{MessageHeaders, MessagePublisher};
use product_contracts::v1::{ProductEvent, EVENT_NAME, EVENT_VERSION};
use shared::errors::api_error::ApiError;
use shared::types::PageRequest;
use tsid::TSID;

pub async fn create_product(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
    Json(payload): Json<CreateProductDto>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let request: CreateProductRequest = payload.try_into()?;
    let product = state
        .create_product
        .execute(request)
        .await
        .map_err(map_api_error)?;
    let event = product_added_event(&product);
    publish_message(state, &event).await?;
    Ok((StatusCode::CREATED, Json(ProductResponseDto::from(product))))
}

pub async fn update_product(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
    Json(payload): Json<UpdateProductDto>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let request = payload.into_request(ProductId::new(id))?;
    let product = state
        .update_product
        .execute(request)
        .await
        .map_err(map_api_error)?;
    let event = product_updated_event(&product);
    publish_message(state, &event).await?;
    Ok((StatusCode::OK, Json(ProductResponseDto::from(product))))
}

pub async fn delete_product(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
    Query(params): Query<DeleteProductParams>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let product_id = ProductId::new(id);
    state
        .delete_product
        .execute(DeleteProductRequest {
            id: product_id,
            updated_by: params.updated_by,
        })
        .await
        .map_err(map_api_error)?;
    let event = ProductEvent::delete(product_id.to_string());
    publish_message(state, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_product(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let product = state
        .get_product
        .execute(GetProductByIdRequest {
            id: ProductId::new(id),
        })
        .await
        .map_err(map_api_error)?;
    Ok((StatusCode::OK, Json(ProductResponseDto::from(product))))
}

pub async fn find_products(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
    Query(params): Query<FindProductsParams>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let organization_id = params
        .organization_id
        .as_deref()
        .map(parse_id)
        .transpose()?;
    let page = state
        .find_products
        .execute(FindProductsRequest {
            name: params.name,
            organization_id,
            page_request: PageRequest::new(
                params.page.unwrap_or(1),
                params.page_size.unwrap_or(20),
            ),
        })
        .await
        .map_err(map_api_error)?
        .map(ProductResponseDto::from);
    Ok((StatusCode::OK, Json(page)))
}

async fn publish_message(state: AppState, event: &ProductEvent) -> Result<(), ApiError> {
    if let Some(publisher) = state.message_publisher.as_ref() {
        publisher
            .publish_json(
                PRODUCT_OUTPUT_EXCHANGE,
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

fn product_added_event(product: &crate::domain::Product) -> ProductEvent {
    ProductEvent::add(
        product.id().to_string(),
        product.organization_id().to_string(),
        product.name().to_string(),
        product.description().map(String::from),
        product.sku().map(String::from),
        product.price(),
    )
}

fn product_updated_event(product: &crate::domain::Product) -> ProductEvent {
    ProductEvent::update(
        product.id().to_string(),
        product.organization_id().to_string(),
        product.name().to_string(),
        product.description().map(String::from),
        product.sku().map(String::from),
        product.price(),
    )
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "UP",
            "service": "product-api",
            "server": {
                "host": state.config.server.host,
                "port": state.config.server.port
            }
        })),
    )
}
