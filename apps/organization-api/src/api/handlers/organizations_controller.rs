use crate::api::add_organization_dtos::CreateOrganizationDto;
use crate::api::delete_organization_dtos::DeleteOrganizationParams;
use crate::api::organization_shared_dtos::OrganizationResponseDto;
use crate::api::state::AppState;
use crate::api::update_organization_dtos::UpdateOrganizationDto;
use crate::application::FindOrganizationByIdRequest;
use crate::application::addusecase::data::AddOrganizationRequest;
use crate::application::deleteusecase::data::DeleteOrganizationRequest;
use crate::application::updateusecase::data::UpdateOrganizationRequest;
use crate::domain::OrganizationId;
use axum::extract::{Path, Query, State};
use axum::{Json, http::StatusCode, response::IntoResponse};
use shared::errors::api_error::ApiError;
use tsid::TSID;

pub async fn create_organization(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationDto>,
) -> Result<impl IntoResponse, ApiError> {
    let request: AddOrganizationRequest = payload.try_into()?;
    let organization = state.add_organization_use_case.execute(request).await?;
    let response = OrganizationResponseDto::from(organization);
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
    let response = OrganizationResponseDto::from(organization);
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
    Ok(StatusCode::OK)
}

pub async fn getbyid_organization(
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
