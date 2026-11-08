pub mod in_memory_client_repository;
pub mod postgres_client_repository;

pub use in_memory_client_repository::InMemoryClientRepository;
pub use postgres_client_repository::PostgresClientRepository;
