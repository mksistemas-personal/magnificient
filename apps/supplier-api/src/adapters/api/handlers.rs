use crate::adapters::api::supplier_dto::{
    SupplierResponseDto, CreateSupplierDto, DeleteSupplierParams, FindSuppliersParams, UpdateSupplierDto,
    map_api_error, parse_id,
};
use crate::adapters::api::state::AppState;
use crate::application::{
    CreateSupplierRequest, DeleteSupplierRequest, FindSuppliersRequest, GetSupplierByIdRequest,
};
use crate::constants::{SUPPLIER_OUTPUT_EXCHANGE, EVENT_SOURCE, ROUTING_KEY};
use crate::domain::SupplierId;
use crate::infrastructure::tenant_state;
use axum::extract::{Path, Query, State};
use axum::{Json, http::StatusCode, response::IntoResponse};
use supplier_contracts::v1::{SupplierAddress, SupplierEvent, EVENT_NAME, EVENT_VERSION};
use messaging::{MessageHeaders, MessagePublisher};
use shared::entities::Address;
use shared::errors::api_error::ApiError;
use shared::types::PageRequest;
use tsid::TSID;

pub async fn create_supplier(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
    Json(payload): Json<CreateSupplierDto>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let request: CreateSupplierRequest = payload.try_into()?;
    let supplier = state
        .create_supplier
        .execute(request)
        .await
        .map_err(map_api_error)?;
    let event = supplier_added_event(&supplier);
    publish_message(state, &event).await?;
    Ok((StatusCode::CREATED, Json(SupplierResponseDto::from(supplier))))
}

pub async fn update_supplier(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
    Json(payload): Json<UpdateSupplierDto>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let request = payload.into_request(SupplierId::new(id))?;
    let supplier = state
        .update_supplier
        .execute(request)
        .await
        .map_err(map_api_error)?;
    let event = supplier_updated_event(&supplier);
    publish_message(state, &event).await?;
    Ok((StatusCode::OK, Json(SupplierResponseDto::from(supplier))))
}

pub async fn delete_supplier(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
    Query(params): Query<DeleteSupplierParams>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let supplier_id = SupplierId::new(id);
    state
        .delete_supplier
        .execute(DeleteSupplierRequest {
            id: supplier_id,
            updated_by: params.updated_by,
        })
        .await
        .map_err(map_api_error)?;
    let event = SupplierEvent::delete(supplier_id.to_string());
    publish_message(state, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn publish_message(state: AppState, event: &SupplierEvent) -> Result<(), ApiError> {
    if let Some(publisher) = state.message_publisher.as_ref() {
        publisher
            .publish_json(
                SUPPLIER_OUTPUT_EXCHANGE,
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

fn supplier_added_event(supplier: &crate::domain::Supplier) -> SupplierEvent {
    SupplierEvent::add(
        supplier.id().to_string(),
        supplier.organization_id().to_string(),
        supplier.name().to_string(),
        supplier.nickname().map(String::from),
        supplier.second_address().map(supplier_address_event),
    )
}

fn supplier_updated_event(supplier: &crate::domain::Supplier) -> SupplierEvent {
    SupplierEvent::update(
        supplier.id().to_string(),
        supplier.organization_id().to_string(),
        supplier.name().to_string(),
        supplier.nickname().map(String::from),
        supplier.second_address().map(supplier_address_event),
    )
}

fn supplier_address_event(address: &Address) -> SupplierAddress {
    SupplierAddress {
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

pub async fn get_supplier(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let supplier = state
        .get_supplier
        .execute(GetSupplierByIdRequest {
            id: SupplierId::new(id),
        })
        .await
        .map_err(map_api_error)?;
    Ok((StatusCode::OK, Json(SupplierResponseDto::from(supplier))))
}

pub async fn find_suppliers(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
    Query(params): Query<FindSuppliersParams>,
) -> Result<impl IntoResponse, ApiError> {
    let state = tenant_state(&state, &tenant).await?;
    let organization_id = params
        .organization_id
        .as_deref()
        .map(parse_id)
        .transpose()?;
    let page = state
        .find_suppliers
        .execute(FindSuppliersRequest {
            name: params.name,
            organization_id,
            page_request: PageRequest::new(
                params.page.unwrap_or(1),
                params.page_size.unwrap_or(20),
            ),
        })
        .await
        .map_err(map_api_error)?
        .map(SupplierResponseDto::from);
    Ok((StatusCode::OK, Json(page)))
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "UP",
            "service": "supplier-api",
            "server": {
                "host": state.config.server.host,
                "port": state.config.server.port
            }
        })),
    )
}
