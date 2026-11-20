use crate::adapters::api::product_dto::{
    ProductResponseDto, CreateProductDto, DeleteProductParams, FindProductsParams, UpdateProductDto,
    map_api_error, parse_id,
};
use crate::adapters::api::state::AppState;
use crate::application::{
    CreateProductRequest, DeleteProductRequest, FindProductsRequest, GetProductByIdRequest,
};
use crate::domain::ProductId;
use crate::infrastructure::tenant_state;
use axum::extract::{Path, Query, State};
use axum::{Json, http::StatusCode, response::IntoResponse};
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
