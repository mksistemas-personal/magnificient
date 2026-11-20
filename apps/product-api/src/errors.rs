use shared::entities::BusinessError;
use tsid::TSID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProductBusinessErrors;

impl ProductBusinessErrors {
    pub const NOT_FOUND: &'static str = "product.not.found";
    pub const NAME_EMPTY: &'static str = "product.name.empty";
    pub const CREATOR_REQUIRED: &'static str = "product.creator.required";
    pub const UPDATER_REQUIRED: &'static str = "product.updater.required";
    pub const ORGANIZATION_REQUIRED: &'static str = "product.organization.required";
    pub const ORGANIZATION_ID_INVALID: &'static str = "product.organization_id.invalid";
    pub const ORGANIZATION_NOT_FOUND: &'static str = "product.organization.not.found";
    pub const PAGINATION_INVALID: &'static str = "product.pagination.invalid";
    pub const PERSIST_FAILED: &'static str = "product.persist.failed";
    pub const QUERY_FAILED: &'static str = "product.query.failed";
    pub const DELETE_FAILED: &'static str = "product.delete.failed";

    pub fn not_found(id: TSID) -> BusinessError {
        BusinessError::not_found(format!("{}: {id}", Self::NOT_FOUND))
    }

    pub fn name_empty() -> BusinessError {
        BusinessError::invalid_data(Self::NAME_EMPTY)
    }

    pub fn creator_required() -> BusinessError {
        BusinessError::invalid_data(Self::CREATOR_REQUIRED)
    }

    pub fn updater_required() -> BusinessError {
        BusinessError::invalid_data(Self::UPDATER_REQUIRED)
    }

    pub fn organization_required() -> BusinessError {
        BusinessError::invalid_data(Self::ORGANIZATION_REQUIRED)
    }

    pub fn organization_not_found() -> BusinessError {
        BusinessError::not_found(Self::ORGANIZATION_NOT_FOUND)
    }

    pub fn pagination_invalid() -> BusinessError {
        BusinessError::invalid_data(Self::PAGINATION_INVALID)
    }

    pub fn persist_failed(error: impl std::fmt::Display) -> BusinessError {
        BusinessError::internal(format!("{}: {error}", Self::PERSIST_FAILED))
    }

    pub fn query_failed(error: impl std::fmt::Display) -> BusinessError {
        BusinessError::internal(format!("{}: {error}", Self::QUERY_FAILED))
    }

    pub fn delete_failed(error: impl std::fmt::Display) -> BusinessError {
        BusinessError::internal(format!("{}: {error}", Self::DELETE_FAILED))
    }
}
