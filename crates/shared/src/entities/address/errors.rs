use crate::entities::BusinessError;
use crate::errors::AddressError;

/// Centralizes message constants and error constructors for the address domain,
/// covering validation (`AddressError`) and persistence (`BusinessError`) errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AddressBusinessErrors;

impl AddressBusinessErrors {
    // -----------------------------------------------------------------------
    // Required field constants
    // -----------------------------------------------------------------------

    pub const FIELD_STREET: &'static str = "street";
    pub const FIELD_NUMBER: &'static str = "number";
    pub const FIELD_NEIGHBORHOOD: &'static str = "neighborhood";
    pub const FIELD_CITY: &'static str = "city";
    pub const FIELD_STATE: &'static str = "state";
    pub const FIELD_STATE_CODE: &'static str = "state_code";
    pub const FIELD_POSTAL_CODE: &'static str = "postal_code";
    pub const FIELD_COUNTRY_CODE: &'static str = "country_code";

    // -----------------------------------------------------------------------
    // Expected postal code formats by country
    // -----------------------------------------------------------------------

    pub const POSTAL_CODE_EXPECTED_BR: &'static str =
        "8 numeric digits (e.g., 01310-100 or 01310100)";
    pub const POSTAL_CODE_EXPECTED_AR: &'static str =
        "4 numeric digits (e.g., 1425) or an 8-character CPA (e.g., C1024CWN)";
    pub const POSTAL_CODE_EXPECTED_US: &'static str =
        "5 digits (e.g., 90210) or ZIP+4 (e.g., 90210-1234)";

    // -----------------------------------------------------------------------
    // Persistence error constants
    // -----------------------------------------------------------------------

    pub const NOT_FOUND: &'static str = "address.not.found";
    pub const PERSIST_FAILED: &'static str = "address.persist.failed";
    pub const QUERY_FAILED: &'static str = "address.query.failed";
    pub const DELETE_FAILED: &'static str = "address.delete.failed";

    // -----------------------------------------------------------------------
    // Validation error constructors (`AddressError`)
    // -----------------------------------------------------------------------

    pub fn empty_street() -> AddressError {
        AddressError::EmptyField(Self::FIELD_STREET)
    }

    pub fn empty_number() -> AddressError {
        AddressError::EmptyField(Self::FIELD_NUMBER)
    }

    pub fn empty_neighborhood() -> AddressError {
        AddressError::EmptyField(Self::FIELD_NEIGHBORHOOD)
    }

    pub fn empty_city() -> AddressError {
        AddressError::EmptyField(Self::FIELD_CITY)
    }

    pub fn empty_state() -> AddressError {
        AddressError::EmptyField(Self::FIELD_STATE)
    }

    pub fn empty_state_code() -> AddressError {
        AddressError::EmptyField(Self::FIELD_STATE_CODE)
    }

    pub fn empty_postal_code() -> AddressError {
        AddressError::EmptyField(Self::FIELD_POSTAL_CODE)
    }

    pub fn empty_country_code() -> AddressError {
        AddressError::EmptyField(Self::FIELD_COUNTRY_CODE)
    }

    pub fn invalid_state_code(
        country: impl Into<String>,
        received: impl Into<String>,
    ) -> AddressError {
        AddressError::InvalidStateCode {
            country: country.into(),
            received: received.into(),
        }
    }

    pub fn invalid_postal_code_br() -> AddressError {
        AddressError::InvalidPostalCode {}
    }

    pub fn invalid_postal_code_ar() -> AddressError {
        AddressError::InvalidPostalCode {}
    }

    pub fn invalid_postal_code_us() -> AddressError {
        AddressError::InvalidPostalCode {}
    }

    // -----------------------------------------------------------------------
    // Persistence error constructors (`BusinessError`)
    // -----------------------------------------------------------------------

    pub fn not_found() -> BusinessError {
        BusinessError::not_found(Self::NOT_FOUND)
    }

    pub fn persist_failed() -> BusinessError {
        BusinessError::internal(Self::PERSIST_FAILED)
    }

    pub fn query_failed() -> BusinessError {
        BusinessError::internal(Self::QUERY_FAILED)
    }

    pub fn delete_failed() -> BusinessError {
        BusinessError::internal(Self::DELETE_FAILED)
    }
}
