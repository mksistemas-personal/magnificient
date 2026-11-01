#[allow(clippy::module_inception)]
pub mod address;
pub mod repository;
pub mod validator;

pub use address::Address;
pub use repository::{AddressRepository, InMemoryAddressRepository, PostgresAddressRepository};
pub use validator::{
    validator_for_country, AddressValidator, ArgentinaAddressValidator, BrazilAddressValidator,
    GenericAddressValidator, UnitedStatesAddressValidator,
};
