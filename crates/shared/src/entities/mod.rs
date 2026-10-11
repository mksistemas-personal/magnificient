// Entidades compartilhadas

pub mod audit;
pub mod document;
pub mod identifier;
pub mod use_case;

pub use audit::audit_fields::AuditFields;
pub use document::{Country, Document, DocumentType};
pub use identifier::{EntityId, Id, TSID};
pub use use_case::business_error::BusinessError;
pub use use_case::use_case_traits::{AddEntityUseCase, UseCase};
