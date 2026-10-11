use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use shared::entities::AddEntityUseCase;
use crate::api::dtos::{CreateOrganizationDto, OrganizationResponseDto};
use crate::api::errors::ApiError;
use crate::api::state::AppState;
use crate::application::add_organization::AddOrganizationRequest;

pub async fn create_organization(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationDto>,
) -> Result<impl IntoResponse, ApiError> {
    let request: AddOrganizationRequest = payload.try_into()?;
    let organization = state.add_organization_use_case.execute(request)?;
    let response = OrganizationResponseDto::from(organization);
    Ok((StatusCode::CREATED, Json(response)))
}
