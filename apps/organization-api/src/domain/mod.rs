// Modelos e regras de domínio

pub mod another_ids;
pub mod organization;
pub mod organization_link;
pub mod organization_repository;

pub use another_ids::ClientId;
pub use organization::{Organization, OrganizationId};
pub use organization_link::{OrganizationEntityType, OrganizationLink};
pub use organization_repository::OrganizationRepository;
pub use shared::entities::AddressRepository;
