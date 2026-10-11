use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use shared::entities::BusinessError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    UnprocessableEntity(String),
    Forbidden(String),
    Internal(String),
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::UnprocessableEntity(_) => "UNPROCESSABLE_ENTITY",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::Internal(_) => "INTERNAL_SERVER_ERROR",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::BadRequest(msg)
            | Self::NotFound(msg)
            | Self::Conflict(msg)
            | Self::UnprocessableEntity(msg)
            | Self::Forbidden(msg)
            | Self::Internal(msg) => msg.as_str(),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_code(), self.message())
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(ErrorResponse {
            error: self.message().to_string(),
            code: self.error_code().to_string(),
            status: status.as_u16(),
            details: None,
        });

        (status, body).into_response()
    }
}

impl From<BusinessError> for ApiError {
    fn from(err: BusinessError) -> Self {
        match err {
            BusinessError::NotFound(msg) => ApiError::NotFound(msg),
            BusinessError::AlreadyExists(msg) => ApiError::Conflict(msg),
            BusinessError::InvalidData(msg) => ApiError::BadRequest(msg),
            BusinessError::Validation(msg) => ApiError::UnprocessableEntity(msg),
            BusinessError::NotAllowed(msg) => ApiError::Forbidden(msg),
            BusinessError::Internal(msg) => ApiError::Internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_error_conversion_to_api_error() {
        let not_found = BusinessError::not_found("Org 1");
        let api_err: ApiError = not_found.into();
        assert_eq!(api_err.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(api_err.error_code(), "NOT_FOUND");
        assert_eq!(api_err.message(), "Org 1");

        let conflict = BusinessError::already_exists("Org duplicada");
        let api_err: ApiError = conflict.into();
        assert_eq!(api_err.status_code(), StatusCode::CONFLICT);
        assert_eq!(api_err.error_code(), "CONFLICT");

        let invalid = BusinessError::invalid_data("Documento invalido");
        let api_err: ApiError = invalid.into();
        assert_eq!(api_err.status_code(), StatusCode::BAD_REQUEST);

        let validation = BusinessError::validation("Regra falhou");
        let api_err: ApiError = validation.into();
        assert_eq!(api_err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

        let not_allowed = BusinessError::not_allowed("Nao permitido");
        let api_err: ApiError = not_allowed.into();
        assert_eq!(api_err.status_code(), StatusCode::FORBIDDEN);

        let internal = BusinessError::internal("Falha de IO");
        let api_err: ApiError = internal.into();
        assert_eq!(api_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
