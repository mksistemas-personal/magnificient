use crate::unit_of_work::UnitOfWork;
use shared::entities::BusinessError;
use sqlx::{PgConnection, PgPool, Postgres, Transaction};
use std::future::Future;
use std::pin::Pin;

type ConnFuture<'c, T> = Pin<Box<dyn Future<Output = Result<T, BusinessError>> + Send + 'c>>;

/// Executa operações de banco de dados com ou sem transação, a partir de um `PgPool`
/// e de um `UnitOfWork` opcional.
///
/// Centraliza a lógica de decisão transacional, eliminando o padrão
/// `match unit_of_work { Some(uow) => ..., None => ... }` em cada método de persistência.
///
/// # Casos de uso
///
/// - **Com `UnitOfWork`**: abre uma transação gerenciada via `uow.transaction()`.
///   Se o `UnitOfWork` foi configurado com `with_schema()`, o schema do tenant é
///   aplicado automaticamente antes de qualquer operação.
/// - **Com pool (`Some(PgPool)`)**: adquire uma conexão do pool sem transação explícita.
/// - **Sem pool (`None`)**: passa um ponteiro de conexão inerte, adequado exclusivamente
///   para repositórios in-memory que ignoram o `&mut PgConnection` (testes unitários).
#[derive(Clone)]
pub struct ConnectionExecutor {
    pool: Option<PgPool>,
    unit_of_work: Option<UnitOfWork>,
}

impl ConnectionExecutor {
    /// Cria um executor com pool real. Adequado para produção sem transação gerenciada.
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Some(pool),
            unit_of_work: None,
        }
    }

    /// Cria um executor sem pool, para uso exclusivo em testes unitários com repositórios
    /// in-memory que ignoram completamente o `&mut PgConnection`.
    pub fn new_without_pool() -> Self {
        Self {
            pool: None,
            unit_of_work: None,
        }
    }

    /// Associa um `UnitOfWork` a este executor.
    /// Quando presente, todas as operações via `run()` ocorrem dentro de uma transação
    /// gerenciada pelo UoW (incluindo aplicação de schema de tenant, se configurado).
    pub fn with_unit_of_work(mut self, unit_of_work: UnitOfWork) -> Self {
        self.unit_of_work = Some(unit_of_work);
        self
    }

    /// Retorna o `UnitOfWork` associado, se houver.
    pub fn unit_of_work(&self) -> Option<&UnitOfWork> {
        self.unit_of_work.as_ref()
    }

    /// Retorna o pool associado, se houver.
    pub fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    /// Executa um callback que recebe `&mut PgConnection`, selecionando a fonte de
    /// conexão de acordo com a configuração:
    ///
    /// 1. **Com `UnitOfWork`** → `uow.transaction(|tx| f(&mut **tx))`
    /// 2. **Com pool** → `pool.acquire()` + `f(&mut conn)`
    /// 3. **Sem pool** → ponteiro inerte (apenas in-memory / testes)
    pub async fn run<T, F>(&self, f: F) -> Result<T, BusinessError>
    where
        T: Send + 'static,
        F: for<'c> FnOnce(&'c mut PgConnection) -> ConnFuture<'c, T> + Send + 'static,
    {
        match &self.unit_of_work {
            Some(uow) => {
                uow.transaction(move |tx: &mut Transaction<'static, Postgres>| {
                    // SAFETY: `conn` é um reborrow de `tx` via Deref. O cast de lifetime
                    // é necessário para satisfazer o `for<'c>` do callback, mas o Future
                    // retornado por `f(conn)` não escapa do escopo de `tx` — o UoW mantém
                    // `tx` vivo até o Future completar antes de fazer commit/rollback.
                    let conn: &mut PgConnection = unsafe { &mut *(&mut **tx as *mut PgConnection) };
                    f(conn)
                })
                .await
            }
            None => match &self.pool {
                Some(pool) => {
                    let mut conn = pool.acquire().await.map_err(|e| {
                        BusinessError::internal(format!("Failed to acquire connection: {e}"))
                    })?;
                    f(&mut conn).await
                }
                None => {
                    // Sem pool e sem UoW: ponteiro inerte para repositórios in-memory.
                    // SAFETY: o ponteiro nunca é desreferenciado pelo in-memory repository.
                    let conn: &mut PgConnection =
                        unsafe { &mut *std::ptr::NonNull::dangling().as_ptr() };
                    f(conn).await
                }
            },
        }
    }
}

impl From<PgPool> for ConnectionExecutor {
    fn from(pool: PgPool) -> Self {
        Self::new(pool)
    }
}

impl From<(PgPool, UnitOfWork)> for ConnectionExecutor {
    fn from((pool, uow): (PgPool, UnitOfWork)) -> Self {
        Self::new(pool).with_unit_of_work(uow)
    }
}
