use crate::application::persistence::OrganizationPersistence;
use crate::domain::OrganizationRepository;
use database::UnitOfWork;
use shared::entities::AddressRepository;
use sqlx::PgPool;

/// Provê construtores padrão para todos os serviços de caso de uso que dependem de
/// `OrganizationPersistence`. Implementar apenas `with_persistence` é suficiente para
/// herdar todos os demais construtores.
pub trait FromPersistence: Sized {
    fn with_persistence(persistence: OrganizationPersistence) -> Self;

    fn new(pool: PgPool, repository: impl OrganizationRepository + 'static) -> Self {
        Self::with_persistence(OrganizationPersistence::new(repository, pool))
    }

    /// Construtor para uso em testes unitários com repositório in-memory.
    /// Não requer pool de banco de dados.
    fn new_for_tests(repository: impl OrganizationRepository + 'static) -> Self {
        Self::with_persistence(OrganizationPersistence::new_without_pool(repository))
    }

    fn new_with_repositories(
        pool: PgPool,
        repository: impl OrganizationRepository + 'static,
        address_repository: impl AddressRepository + 'static,
    ) -> Self {
        Self::with_persistence(
            OrganizationPersistence::new(repository, pool)
                .with_address_repository(address_repository),
        )
    }

    fn new_with_repositories_for_tests(
        repository: impl OrganizationRepository + 'static,
        address_repository: impl AddressRepository + 'static,
    ) -> Self {
        Self::with_persistence(
            OrganizationPersistence::new_without_pool(repository)
                .with_address_repository(address_repository),
        )
    }

    fn new_with_repositories_and_uow(
        pool: PgPool,
        repository: impl OrganizationRepository + 'static,
        address_repository: impl AddressRepository + 'static,
        uow: UnitOfWork,
    ) -> Self {
        Self::with_persistence(
            OrganizationPersistence::new(repository, pool)
                .with_address_repository(address_repository)
                .with_unit_of_work(uow),
        )
    }

    fn new_with_uow(
        pool: PgPool,
        repository: impl OrganizationRepository + 'static,
        uow: UnitOfWork,
    ) -> Self {
        Self::with_persistence(
            OrganizationPersistence::new(repository, pool).with_unit_of_work(uow),
        )
    }
}
