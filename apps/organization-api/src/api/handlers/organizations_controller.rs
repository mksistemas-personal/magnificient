use crate::api::add_organization_dtos::CreateOrganizationDto;
use crate::api::organization_shared_dtos::OrganizationResponseDto;
use crate::api::state::AppState;
use crate::application::add_organization_service::AddOrganizationRequest;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use shared::errors::api_error::ApiError;

pub async fn create_organization(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationDto>,
) -> Result<impl IntoResponse, ApiError> {
    let request: AddOrganizationRequest = payload.try_into()?;
    let organization = state.add_organization_use_case.execute(request).await?;
    let response = OrganizationResponseDto::from(organization);
    Ok((StatusCode::CREATED, Json(response)))
}
