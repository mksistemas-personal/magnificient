#[allow(clippy::module_inception)]
pub mod address;
pub mod errors;
pub mod repository;
pub mod validator;

pub use address::Address;
pub use errors::AddressBusinessErrors;
pub use repository::{AddressRepository, InMemoryAddressRepository, PostgresAddressRepository};
pub use validator::{
    AddressValidator, ArgentinaAddressValidator, BrazilAddressValidator, GenericAddressValidator,
    UnitedStatesAddressValidator, validator_for_country,
};
