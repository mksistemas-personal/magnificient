use crate::api::add_organization_dto::CreateOrganizationDto;
use crate::api::delete_organization_dto::DeleteOrganizationParams;
use crate::api::find_all_organization_dto::FindAllOrganizationsParams;
use crate::api::organization_shared_dto::OrganizationResponseDto;
use crate::api::state::AppState;
use crate::api::update_organization_dto::UpdateOrganizationDto;
use crate::application::{
    AddOrganizationRequest, DeleteOrganizationRequest, FindAllOrganizationsRequest,
    FindOrganizationByIdRequest, UpdateOrganizationRequest,
};
use crate::domain::OrganizationId;
use crate::domain::message_events::OrganizationEvent;
use axum::extract::{Path, Query, State};
use axum::{Json, http::StatusCode, response::IntoResponse};
use messaging::{MessageHeaders, MessagePublisher};
use shared::errors::api_error::ApiError;
use tsid::TSID;

const ORGANIZATION_EXCHANGE_NAME: &str = "organization.output";
const ROUTING_KEY: &str = "*";

const EVENT_TYPE: &str = "OrganizationEvent";
const EVENT_SOURCE: &str = "organization-api";
const EVENT_VERSION: i32 = 100;

pub async fn create_organization(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationDto>,
) -> Result<impl IntoResponse, ApiError> {
    let request: AddOrganizationRequest = payload.try_into()?;
    let organization = state.add_organization_use_case.execute(request).await?;
    let response = OrganizationResponseDto::from(organization.clone());
    let event = OrganizationEvent::add(organization.clone());
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
    let event = OrganizationEvent::update(organization.clone());
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
    let event = OrganizationEvent::delete(organization_id);
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
                    EVENT_TYPE,
                    EVENT_SOURCE,
                    EVENT_VERSION,
                )),
            )
            .await?;
    }
    Ok(())
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
