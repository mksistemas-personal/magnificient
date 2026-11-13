use crate::api::AppState;
use shared::errors::api_error::ApiError;

/// Resolves an active tenant and creates an `AppState` scoped to its schema.
pub async fn tenant_state(state: &AppState, tenant: &str) -> Result<AppState, ApiError> {
    let valid = !tenant.is_empty()
        && tenant.len() <= 56
        && tenant.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && tenant.ends_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && tenant
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !tenant.contains("--");
    if !valid {
        return Err(ApiError::BadRequest("Invalid tenant identifier".into()));
    }

    let schema: Option<String> = sqlx::query_scalar(
        "SELECT schema_name FROM public.tenants WHERE slug = $1 AND active = TRUE",
    )
    .bind(tenant)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|error| ApiError::Internal(format!("Failed to resolve tenant: {error}")))?;
    let schema = schema.ok_or_else(|| ApiError::NotFound("Tenant not found".into()))?;

    state
        .for_tenant_schema(&schema, tenant)
        .map_err(ApiError::from)
}
