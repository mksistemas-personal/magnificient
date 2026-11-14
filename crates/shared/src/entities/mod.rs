// Entidades compartilhadas

pub mod address;
pub mod audit;
pub mod converter;
pub mod document;
pub mod identifier;
pub mod use_case;

pub use address::{
    Address, AddressBusinessErrors, AddressRepository, InMemoryAddressRepository,
    PostgresAddressRepository,
};
pub use audit::AuditFields;
pub use converter::default_user;
pub use document::{Country, Document, DocumentType};
pub use identifier::{EntityId, Id, TSID};
pub use use_case::{
    AddEntityUseCase, AsyncAddEntityUseCase, AsyncDeleteEntityUseCase, AsyncGetAllUseCase,
    AsyncGetByIdUseCase, AsyncResultExt, AsyncUpdateEntityUseCase, AsyncUseCase, BusinessError,
    Context, UseCase,
};
