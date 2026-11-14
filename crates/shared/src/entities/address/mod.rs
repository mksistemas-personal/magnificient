#[allow(clippy::module_inception)]
mod address;
mod errors;
mod repository;
pub(crate) mod validator;

pub use address::Address;
pub use errors::AddressBusinessErrors;
pub use repository::{AddressRepository, InMemoryAddressRepository, PostgresAddressRepository};
