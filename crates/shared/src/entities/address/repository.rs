use crate::entities::address::Address;
use crate::entities::AddressBusinessErrors;
use crate::entities::{AuditFields, BusinessError};
use async_trait::async_trait;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::error;
use tsid::{TSID, create_tsid};

/// Trait que define o contrato de persistência para a entidade `Address`.
#[async_trait]
pub trait AddressRepository: Send + Sync {
    /// Saves or inserts an address in the database.
    /// Generates a new TSID if the address does not have an ID.
    async fn save(&self, address: &Address) -> Result<Address, BusinessError>;

    /// Saves an address with an explicit TSID and optional audit fields.
    async fn save_with_id(
        &self,
        id: TSID,
        address: &Address,
        audit: Option<&AuditFields>,
    ) -> Result<Address, BusinessError>;

    /// Finds an address by its TSID.
    async fn find_by_id(&self, id: TSID) -> Result<Address, BusinessError>;

    /// Soft-deletes an address by its TSID.
    async fn delete(&self, id: TSID, updated_by: &str) -> Result<(), BusinessError>;

    async fn save_in_tx(
        &self,
        conn: &mut sqlx::PgConnection,
        address: &Address,
    ) -> Result<Address, BusinessError>;
}

/// In-memory implementation of the `Address` repository, suitable for unit and integration tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAddressRepository {
    items: Arc<RwLock<HashMap<TSID, Address>>>,
}

impl InMemoryAddressRepository {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn len(&self) -> usize {
        self.items.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.read().unwrap().is_empty()
    }

    pub fn get(&self, id: TSID) -> Option<Address> {
        self.items.read().unwrap().get(&id).cloned()
    }

    pub fn clear(&self) {
        self.items.write().unwrap().clear();
    }
}

#[async_trait]
impl AddressRepository for InMemoryAddressRepository {
    async fn save(&self, address: &Address) -> Result<Address, BusinessError> {
        let id = address.id().unwrap_or_else(create_tsid);
        let mut saved_address = address.clone();
        saved_address.id = Some(id);
        if saved_address.audit.is_none() {
            saved_address.audit = Some(AuditFields::created("system"));
        }

        let mut map = self.items.write().unwrap();
        map.insert(id, saved_address.clone());
        Ok(saved_address)
    }

    async fn save_with_id(
        &self,
        id: TSID,
        address: &Address,
        audit: Option<&AuditFields>,
    ) -> Result<Address, BusinessError> {
        let mut saved_address = address.clone();
        saved_address.id = Some(id);
        if let Some(audit_fields) = audit {
            saved_address.audit = Some(audit_fields.clone());
        } else if saved_address.audit.is_none() {
            saved_address.audit = Some(AuditFields::created("system"));
        }

        let mut map = self.items.write().unwrap();
        map.insert(id, saved_address.clone());
        Ok(saved_address)
    }

    async fn find_by_id(&self, id: TSID) -> Result<Address, BusinessError> {
        let map = self.items.read().unwrap();
        map.get(&id)
            .cloned()
            .ok_or_else(|| AddressBusinessErrors::not_found())
    }

    async fn delete(&self, id: TSID, _updated_by: &str) -> Result<(), BusinessError> {
        let mut map = self.items.write().unwrap();
        if map.remove(&id).is_none() {
            return Err(AddressBusinessErrors::not_found());
        }
        Ok(())
    }

    async fn save_in_tx(
        &self,
        _conn: &mut sqlx::PgConnection,
        address: &Address,
    ) -> Result<Address, BusinessError> {
        // The in-memory adapter has no database transaction or rollback semantics.
        self.save(address).await
    }
}

/// Represents an address row returned by PostgreSQL.
#[derive(sqlx::FromRow)]
struct AddressRow {
    id: i64,
    street: String,
    number: String,
    complement: Option<String>,
    neighborhood: Option<String>,
    city: String,
    state: Option<String>,
    state_code: Option<String>,
    postal_code: String,
    country: Option<String>,
    country_code: Option<String>,
    reference: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    created_by: String,
    updated_by: String,
}

impl From<AddressRow> for Address {
    fn from(row: AddressRow) -> Self {
        Address::new(
            row.street,
            row.number,
            row.complement.unwrap_or_default(),
            row.neighborhood.unwrap_or_default(),
            row.city,
            row.state.unwrap_or_default(),
            row.state_code.unwrap_or_default(),
            row.postal_code,
            row.country.unwrap_or_default(),
            row.country_code.unwrap_or_default(),
            row.reference.unwrap_or_default(),
        )
        .with_id(TSID::from(row.id as u64))
        .with_audit(AuditFields {
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
            updated_by: row.updated_by,
        })
    }
}

/// PostgreSQL and SQLx implementation of the `Address` repository.
#[derive(Debug, Clone)]
pub struct PostgresAddressRepository {
    pool: PgPool,
}

impl PostgresAddressRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn from_pool(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn execute_save<'e, E>(executor: E, address: &Address) -> Result<Address, BusinessError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let id = address.id().unwrap_or_else(create_tsid);
        let id_raw = id.number() as i64;

        let (created_at, updated_at, created_by, updated_by): (
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
            String,
            String,
        ) = match &address.audit {
            Some(audit) => (
                audit.created_at(),
                audit.updated_at(),
                audit.created_by().to_string(),
                audit.updated_by().to_string(),
            ),
            None => {
                let now = chrono::Utc::now();
                (now, now, "system".to_string(), "system".to_string())
            }
        };

        let complement = if address.complement.is_empty() {
            None
        } else {
            Some(address.complement.as_str())
        };
        let neighborhood = if address.neighborhood.is_empty() {
            None
        } else {
            Some(address.neighborhood.as_str())
        };
        let state = if address.state.is_empty() {
            None
        } else {
            Some(address.state.as_str())
        };
        let state_code = if address.state_code.is_empty() {
            None
        } else {
            Some(address.state_code.as_str())
        };
        let country = if address.country.is_empty() {
            None
        } else {
            Some(address.country.as_str())
        };
        let country_code = if address.country_code.is_empty() {
            None
        } else {
            Some(address.country_code.as_str())
        };
        let reference = if address.reference.is_empty() {
            None
        } else {
            Some(address.reference.as_str())
        };

        let query = r#"
            INSERT INTO addresses (
                id, street, number, complement, neighborhood, city, state, state_code,
                postal_code, country, country_code, reference, deleted,
                created_at, updated_at, created_by, updated_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT (id) DO UPDATE SET
                street = EXCLUDED.street,
                number = EXCLUDED.number,
                complement = EXCLUDED.complement,
                neighborhood = EXCLUDED.neighborhood,
                city = EXCLUDED.city,
                state = EXCLUDED.state,
                state_code = EXCLUDED.state_code,
                postal_code = EXCLUDED.postal_code,
                country = EXCLUDED.country,
                country_code = EXCLUDED.country_code,
                reference = EXCLUDED.reference,
                deleted = EXCLUDED.deleted,
                updated_at = EXCLUDED.updated_at,
                updated_by = EXCLUDED.updated_by
        "#;

        sqlx::query(query)
            .bind(id_raw)
            .bind(&address.street)
            .bind(&address.number)
            .bind(complement)
            .bind(neighborhood)
            .bind(&address.city)
            .bind(state)
            .bind(state_code)
            .bind(&address.postal_code)
            .bind(country)
            .bind(country_code)
            .bind(reference)
            .bind(false)
            .bind(created_at)
            .bind(updated_at)
            .bind(&created_by)
            .bind(&updated_by)
            .execute(executor)
            .await
            .map_err(|err| {
                error!("Failed to save address to the database: {:?}", err);
                AddressBusinessErrors::persist_failed()
            })?;

        let mut saved = address.clone();
        saved.id = Some(id);
        saved.audit = Some(AuditFields {
            created_at,
            updated_at,
            created_by,
            updated_by,
        });

        Ok(saved)
    }

    async fn execute_find_by_id<'e, E>(executor: E, id: TSID) -> Result<Address, BusinessError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let id_raw = id.number() as i64;
        let query = r#"
            SELECT id, street, number, complement, neighborhood, city, state, state_code,
                   postal_code, country, country_code, reference,
                   created_at, updated_at, created_by, updated_by
            FROM addresses
            WHERE id = $1 AND deleted = FALSE
        "#;

        let row_opt = sqlx::query_as::<_, AddressRow>(query)
            .bind(id_raw)
            .fetch_optional(executor)
            .await
            .map_err(|err| {
                error!("Failed to fetch address by ID {}: {:?}", id, err);
                AddressBusinessErrors::query_failed()
            })?;

        row_opt
            .map(Address::from)
            .ok_or_else(|| AddressBusinessErrors::not_found())
    }

    async fn execute_delete<'e, E>(
        executor: E,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let id_raw = id.number() as i64;
        let now = chrono::Utc::now();
        let query = r#"
            UPDATE addresses
            SET deleted = TRUE,
                updated_at = $2,
                updated_by = $3
            WHERE id = $1 AND deleted = FALSE
        "#;

        let result = sqlx::query(query)
            .bind(id_raw)
            .bind(now)
            .bind(updated_by)
            .execute(executor)
            .await
            .map_err(|err| {
                error!("Failed to soft-delete address with ID {}: {:?}", id, err);
                AddressBusinessErrors::delete_failed()
            })?;

        if result.rows_affected() == 0 {
            return Err(AddressBusinessErrors::not_found());
        }

        Ok(())
    }
}

#[async_trait]
impl AddressRepository for PostgresAddressRepository {
    async fn save(&self, address: &Address) -> Result<Address, BusinessError> {
        Self::execute_save(&self.pool, address).await
    }

    async fn save_with_id(
        &self,
        id: TSID,
        address: &Address,
        audit: Option<&AuditFields>,
    ) -> Result<Address, BusinessError> {
        let mut addr = address.clone();
        addr.id = Some(id);
        if let Some(audit_fields) = audit {
            addr.audit = Some(audit_fields.clone());
        }
        Self::execute_save(&self.pool, &addr).await
    }

    async fn find_by_id(&self, id: TSID) -> Result<Address, BusinessError> {
        Self::execute_find_by_id(&self.pool, id).await
    }

    async fn delete(&self, id: TSID, updated_by: &str) -> Result<(), BusinessError> {
        Self::execute_delete(&self.pool, id, updated_by).await
    }

    async fn save_in_tx(
        &self,
        conn: &mut sqlx::PgConnection,
        address: &Address,
    ) -> Result<Address, BusinessError> {
        Self::execute_save(conn, address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_address_repository_save_and_find() {
        let repo = InMemoryAddressRepository::new();
        let address = Address::new(
            "Av. Paulista",
            "1000",
            "Conjunto 101",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "01310-100",
            "Brasil",
            "BR",
            "Perto do metrô",
        );

        let saved = repo.save(&address).await.expect("Deveria salvar");
        assert!(saved.id().is_some());
        let id = saved.id().unwrap();

        let found = repo.find_by_id(id).await.expect("Deveria encontrar");
        assert_eq!(found.street, "Av. Paulista");
        assert_eq!(found.number, "1000");
        assert_eq!(found.city, "São Paulo");
        assert_eq!(found.id(), Some(id));
    }

    #[tokio::test]
    async fn test_in_memory_address_repository_save_with_id() {
        let repo = InMemoryAddressRepository::new();
        let custom_id = TSID::from(999999u64);
        let audit = AuditFields::created("admin_user");
        let address = Address::new(
            "Rua das Flores",
            "123",
            "",
            "Centro",
            "Curitiba",
            "Paraná",
            "PR",
            "80000-000",
            "Brasil",
            "BR",
            "",
        );

        let saved = repo
            .save_with_id(custom_id, &address, Some(&audit))
            .await
            .expect("Deveria salvar");
        assert_eq!(saved.id(), Some(custom_id));
        assert!(saved.audit().is_some());
        assert_eq!(saved.audit().unwrap().created_by(), "admin_user");

        let found = repo.find_by_id(custom_id).await.expect("Deveria encontrar");
        assert_eq!(found.id(), Some(custom_id));
        assert_eq!(found.street, "Rua das Flores");
    }

    #[tokio::test]
    async fn test_in_memory_address_repository_delete() {
        let repo = InMemoryAddressRepository::new();
        let address = Address::new(
            "100 Main St",
            "1",
            "",
            "Downtown",
            "New York",
            "New York",
            "NY",
            "10001",
            "United States",
            "US",
            "",
        );

        let saved = repo.save(&address).await.unwrap();
        let id = saved.id().unwrap();

        let delete_res = repo.delete(id, "admin").await;
        assert!(delete_res.is_ok());

        let find_res = repo.find_by_id(id).await;
        assert!(find_res.is_err());
    }

    #[tokio::test]
    async fn test_in_memory_address_repository_not_found() {
        let repo = InMemoryAddressRepository::new();
        let not_found_id = TSID::from(12345u64);

        let find_res = repo.find_by_id(not_found_id).await;
        assert!(find_res.is_err());

        let delete_res = repo.delete(not_found_id, "admin").await;
        assert!(delete_res.is_err());
    }
}
