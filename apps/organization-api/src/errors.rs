use shared::entities::BusinessError;

/// Centralizes `BusinessError` constants and constructors for the organization domain,
/// standardizing business messages and errors used by use cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OrganizationBusinessErrors;

impl OrganizationBusinessErrors {
    /// Error message for an organization that was not found.
    pub const ORG_NOT_FOUND: &'static str = "organizatgion.not.found";

    /// Error message for a blank organization name.
    pub const ORG_NAME_EMPTY: &'static str = "organization.name.empty";

    /// Error message for an organization name that exceeds the maximum length.
    pub const ORG_NAME_TOO_LONG: &'static str = "organization.name.greather.limit";

    /// Error message for an organization that already exists / is duplicated.
    pub const ORG_ALREADY_EXISTS: &'static str = "organization.already.exists";

    /// Error message for an invalid organization tax document.
    pub const ORG_DOCUMENT_INVALID: &'static str = "organization.document.invalid";

    /// Error message for a tax document already used by another organization.
    pub const ORG_DOCUMENT_ALREADY_IN_USE: &'static str = "organization.document.already.in.use";

    /// Error message for an invalid organization identifier.
    pub const ORG_ID_INVALID: &'static str = "organization.id.invalid";

    /// Error message for a missing author when creating an organization.
    pub const ORG_CREATOR_REQUIRED: &'static str = "organization.creator.required";

    pub const ORG_UPDATER_REQUIRED: &'static str = "organization.updater.required";

    /// Error message for operations on inactive organizations.
    pub const ORG_INACTIVE: &'static str = "organization.inactive";

    // --- Convenience methods for creating BusinessError instances ---

    /// Returns `BusinessError::NotFound` for an organization ID.
    pub fn not_found() -> BusinessError {
        BusinessError::not_found(Self::ORG_NOT_FOUND)
    }

    /// Returns `BusinessError::InvalidData` when the name is empty.
    pub fn name_empty() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_NAME_EMPTY)
    }

    /// Returns `BusinessError::InvalidData` when the name exceeds the length limit.
    pub fn name_too_long() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_NAME_TOO_LONG)
    }

    /// Returns `BusinessError::AlreadyExists` when the organization already exists.
    pub fn already_exists() -> BusinessError {
        BusinessError::already_exists(Self::ORG_ALREADY_EXISTS)
    }

    /// Returns `BusinessError::InvalidData` when the document is invalid.
    pub fn document_invalid() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_DOCUMENT_INVALID)
    }

    /// Returns `BusinessError::AlreadyExists` when the document is already registered.
    pub fn document_already_in_use() -> BusinessError {
        BusinessError::already_exists(Self::ORG_DOCUMENT_ALREADY_IN_USE)
    }

    /// Returns `BusinessError::InvalidData` when the ID is invalid.
    pub fn id_invalid() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_ID_INVALID)
    }

    /// Returns `BusinessError::InvalidData` when a creator is required.
    pub fn creator_required() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_CREATOR_REQUIRED)
    }

    pub fn updater_required() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_UPDATER_REQUIRED)
    }

    /// Returns `BusinessError::NotAllowed` when the organization is inactive.
    pub fn inactive() -> BusinessError {
        BusinessError::not_allowed(Self::ORG_INACTIVE)
    }
}

/// Convenience alias.
pub type OrganizationErrors = OrganizationBusinessErrors;
