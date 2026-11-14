use crate::adapters::api::add_organization_dto::CreateOrganizationDto;
use crate::adapters::api::delete_organization_dto::DeleteOrganizationParams;
use crate::adapters::api::find_all_organization_dto::FindAllOrganizationsParams;
use crate::adapters::api::organization_shared_dto::OrganizationResponseDto;
use crate::adapters::api::state::AppState;
use crate::adapters::api::update_organization_dto::UpdateOrganizationDto;
use crate::application::{
    AddOrganizationRequest, DeleteOrganizationRequest, FindAllOrganizationsRequest,
    FindOrganizationByIdRequest, UpdateOrganizationRequest,
};
use crate::domain::{Organization, OrganizationId};
use axum::extract::{Path, Query, State};
use axum::{Json, http::StatusCode, response::IntoResponse};
use messaging::{MessageHeaders, MessagePublisher};
use organization_contracts::v1::{
    EVENT_NAME, EVENT_VERSION, OrganizationAddress, OrganizationDocument, OrganizationEvent,
};
use shared::entities::{Address, Document};
use shared::errors::api_error::ApiError;
use tsid::TSID;

/// Resolves the tenant in the control schema before constructing tenant-scoped services.
async fn scoped_state(state: &AppState, tenant: &str) -> Result<AppState, ApiError> {
    let tenant_valid = !tenant.is_empty()
        && tenant.len() <= 56
        && tenant.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && tenant.ends_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && tenant
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !tenant.contains("--");
    if !tenant_valid {
        return Err(ApiError::BadRequest("Invalid tenant identifier".into()));
    }
    let schema: Option<String> = sqlx::query_scalar(
        "SELECT schema_name FROM public.tenants WHERE slug = $1 AND active = TRUE",
    )
    .bind(tenant)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|err| ApiError::Internal(format!("Failed to resolve tenant: {err}")))?;
    let schema = schema.ok_or_else(|| ApiError::NotFound("Tenant not found".into()))?;
    state
        .for_tenant_schema(&schema, tenant)
        .map_err(ApiError::from)
}

pub async fn create_organization_for_tenant(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
    Json(payload): Json<CreateOrganizationDto>,
) -> Result<impl IntoResponse, ApiError> {
    create_organization(State(scoped_state(&state, &tenant).await?), Json(payload)).await
}

pub async fn update_organization_for_tenant(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
    Json(payload): Json<UpdateOrganizationDto>,
) -> Result<impl IntoResponse, ApiError> {
    update_organization(
        State(scoped_state(&state, &tenant).await?),
        Path(id),
        Json(payload),
    )
    .await
}

pub async fn delete_organization_for_tenant(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
    Query(params): Query<DeleteOrganizationParams>,
) -> Result<impl IntoResponse, ApiError> {
    delete_organization(
        State(scoped_state(&state, &tenant).await?),
        Path(id),
        Query(params),
    )
    .await
}

pub async fn get_by_id_organization_for_tenant(
    State(state): State<AppState>,
    Path((tenant, id)): Path<(String, TSID)>,
) -> Result<impl IntoResponse, ApiError> {
    get_by_id_organization(State(scoped_state(&state, &tenant).await?), Path(id)).await
}

pub async fn get_all_organizations_for_tenant(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
    Query(params): Query<FindAllOrganizationsParams>,
) -> Result<impl IntoResponse, ApiError> {
    get_all_organizations(State(scoped_state(&state, &tenant).await?), Query(params)).await
}

const ORGANIZATION_EXCHANGE_NAME: &str = "organization.output";
const ROUTING_KEY: &str = "*";

const EVENT_SOURCE: &str = "organization-api";

pub async fn create_organization(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationDto>,
) -> Result<impl IntoResponse, ApiError> {
    let request: AddOrganizationRequest = payload.try_into()?;
    let organization = state.add_organization_use_case.execute(request).await?;
    let response = OrganizationResponseDto::from(organization.clone());
    let event = organization_added_event(&organization);
    publish_message(state, &event).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_organization(
    State(state): State<AppState>,
    Path(id): Path<TSID>,
    Json(payload): Json<UpdateOrganizationDto>,
) -> Result<impl IntoResponse, ApiError> {
    let organization_id: OrganizationId = id.into();
    let request: UpdateOrganizationRequest = payload.into_request(organization_id);
    let organization = state.update_organization_use_case.execute(request).await?;
    let response = OrganizationResponseDto::from(organization.clone());
    let event = organization_updated_event(&organization);
    publish_message(state, &event).await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn delete_organization(
    State(state): State<AppState>,
    Path(id): Path<TSID>,
    Query(params): Query<DeleteOrganizationParams>,
) -> Result<impl IntoResponse, ApiError> {
    let organization_id: OrganizationId = id.into();
    let request: DeleteOrganizationRequest = params.into_request(organization_id);
    state.delete_organization_use_case.execute(request).await?;
    let event = OrganizationEvent::delete(organization_id.to_string());
    publish_message(state, &event).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn publish_message(state: AppState, event: &OrganizationEvent) -> Result<(), ApiError> {
    if let Some(publisher) = state.message_publisher.as_ref() {
        publisher
            .publish_json(
                ORGANIZATION_EXCHANGE_NAME,
                ROUTING_KEY,
                &event,
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

fn organization_added_event(organization: &Organization) -> OrganizationEvent {
    OrganizationEvent::add(
        organization.id().to_string(),
        organization.name().to_string(),
        organization_document_event(organization.document()),
        organization
            .address()
            .map(organization_address_event),
    )
}

fn organization_updated_event(organization: &Organization) -> OrganizationEvent {
    OrganizationEvent::update(
        organization.id().to_string(),
        organization.name().to_string(),
        organization_document_event(organization.document()),
        organization
            .address()
            .map(organization_address_event),
    )
}

fn organization_document_event(document: &Document) -> OrganizationDocument {
    OrganizationDocument {
        value: document.value.to_string(),
        document_type: document.document_type.code().to_string(),
        country: document.country().map(|country| country.code().to_string()),
    }
}

fn organization_address_event(address: &Address) -> OrganizationAddress {
    OrganizationAddress {
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

pub async fn get_by_id_organization(
    State(state): State<AppState>,
    Path(id): Path<TSID>,
) -> Result<impl IntoResponse, ApiError> {
    let organization_id: OrganizationId = id.into();
    let request: FindOrganizationByIdRequest = FindOrganizationByIdRequest { organization_id };
    let organization = state
        .get_by_id_organization_use_case
        .execute(request)
        .await?;
    let response = OrganizationResponseDto::from(organization);
    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_all_organizations(
    State(state): State<AppState>,
    Query(params): Query<FindAllOrganizationsParams>,
) -> Result<impl IntoResponse, ApiError> {
    let request: FindAllOrganizationsRequest = params.into_request();
    let page = state.get_all_organization_use_case.execute(request).await?;
    let response = page.map(OrganizationResponseDto::from);
    Ok((StatusCode::OK, Json(response)))
}
