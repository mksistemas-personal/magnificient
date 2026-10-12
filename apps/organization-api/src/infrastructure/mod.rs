// Implementações de infraestrutura e adaptadores

pub mod in_memory_organization_repository;
pub mod postgres_organization_repository;

pub use in_memory_organization_repository::InMemoryOrganizationRepository;
pub use postgres_organization_repository::PostgresOrganizationRepository;
