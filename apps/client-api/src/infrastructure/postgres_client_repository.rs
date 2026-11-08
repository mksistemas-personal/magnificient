use crate::domain::{Client, ClientId, ClientRepository, OrganizationId};
use crate::errors::ClientBusinessErrors;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared::entities::{Address, AuditFields, BusinessError};
use shared::types::{Page, PageRequest};
use sqlx::{PgConnection, QueryBuilder};
use tracing::error;
use tsid::TSID;

const CLIENT_SELECT: &str = r#"
    SELECT c.id, c.organization_id, c.name, c.nickname,
           c.deleted, c.created_at, c.updated_at, c.created_by, c.updated_by,
           a.id AS address_id, a.street AS address_street, a.number AS address_number,
           a.complement AS address_complement, a.neighborhood AS address_neighborhood,
           a.city AS address_city, a.state AS address_state, a.state_code AS address_state_code,
           a.postal_code AS address_postal_code, a.country AS address_country,
           a.country_code AS address_country_code, a.reference AS address_reference,
           a.created_at AS address_created_at, a.updated_at AS address_updated_at,
           a.created_by AS address_created_by, a.updated_by AS address_updated_by
    FROM clients c
    LEFT JOIN addresses a ON a.id = c.second_address_id AND a.deleted = FALSE
    WHERE c.deleted = FALSE
"#;

#[derive(sqlx::FromRow)]
struct ClientRow {
    id: i64,
    organization_id: i64,
    name: String,
    nickname: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: String,
    updated_by: String,
    address_id: Option<i64>,
    address_street: Option<String>,
    address_number: Option<String>,
    address_complement: Option<String>,
    address_neighborhood: Option<String>,
    address_city: Option<String>,
    address_state: Option<String>,
    address_state_code: Option<String>,
    address_postal_code: Option<String>,
    address_country: Option<String>,
    address_country_code: Option<String>,
    address_reference: Option<String>,
    address_created_at: Option<DateTime<Utc>>,
    address_updated_at: Option<DateTime<Utc>>,
    address_created_by: Option<String>,
    address_updated_by: Option<String>,
}

impl TryFrom<ClientRow> for Client {
    type Error = BusinessError;

    fn try_from(row: ClientRow) -> Result<Self, Self::Error> {
        let mut client = Client::new(
            ClientId::new(TSID::from(row.id as u64)),
            OrganizationId::new(TSID::from(row.organization_id as u64)),
            row.name,
            row.nickname,
            None,
        )
        .with_audit(AuditFields {
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
            updated_by: row.updated_by,
        });

        if let Some(id) = row.address_id {
            let address = Address::new(
                required(row.address_street, "street")?,
                required(row.address_number, "number")?,
                row.address_complement.unwrap_or_default(),
                row.address_neighborhood.unwrap_or_default(),
                required(row.address_city, "city")?,
                row.address_state.unwrap_or_default(),
                row.address_state_code.unwrap_or_default(),
                required(row.address_postal_code, "postal_code")?,
                row.address_country.unwrap_or_default(),
                row.address_country_code.unwrap_or_default(),
                row.address_reference.unwrap_or_default(),
            )
            .with_id(TSID::from(id as u64))
            .with_audit(AuditFields {
                created_at: required(row.address_created_at, "created_at")?,
                updated_at: required(row.address_updated_at, "updated_at")?,
                created_by: required(row.address_created_by, "created_by")?,
                updated_by: required(row.address_updated_by, "updated_by")?,
            });
            client.second_address = Some(address);
        }

        Ok(client)
    }
}

fn required<T>(value: Option<T>, field: &str) -> Result<T, BusinessError> {
    value.ok_or_else(|| {
        BusinessError::internal(format!(
            "Required field missing from client address: {field}"
        ))
    })
}

#[derive(Debug, Clone, Default)]
pub struct PostgresClientRepository;

impl PostgresClientRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ClientRepository for PostgresClientRepository {
    async fn save(
        &self,
        conn: &mut PgConnection,
        client: &Client,
    ) -> Result<Client, BusinessError> {
        let now = Utc::now();
        let audit = client
            .audit
            .clone()
            .unwrap_or_else(|| AuditFields::created("system"));
        let address_id = client
            .second_address
            .as_ref()
            .and_then(Address::id)
            .map(|id| id.number() as i64);

        sqlx::query(
            r#"
                INSERT INTO clients (
                    id, organization_id, name, nickname, second_address_id,
                    deleted, created_at, updated_at, created_by, updated_by
                ) VALUES ($1, $2, $3, $4, $5, FALSE, $6, $7, $8, $9)
                ON CONFLICT (id) DO UPDATE SET
                    organization_id = EXCLUDED.organization_id,
                    name = EXCLUDED.name,
                    nickname = EXCLUDED.nickname,
                    second_address_id = EXCLUDED.second_address_id,
                    deleted = FALSE,
                    updated_at = EXCLUDED.updated_at,
                    updated_by = EXCLUDED.updated_by
            "#,
        )
        .bind(client.id.number() as i64)
        .bind(client.organization_id.number() as i64)
        .bind(&client.name)
        .bind(&client.nickname)
        .bind(address_id)
        .bind(audit.created_at)
        .bind(now.max(audit.updated_at))
        .bind(&audit.created_by)
        .bind(&audit.updated_by)
        .execute(&mut *conn)
        .await
        .map_err(|err| {
            if let Some(db_error) = err.as_database_error() {
                if db_error.code().as_deref() == Some("23503") {
                    return ClientBusinessErrors::organization_not_found();
                }
            }
            error!("Failed to persist client {}: {err:?}", client.id);
            ClientBusinessErrors::persist_failed(err)
        })?;

        let mut saved = client.clone();
        saved.audit = Some(AuditFields {
            created_at: audit.created_at,
            updated_at: now.max(audit.updated_at),
            created_by: audit.created_by,
            updated_by: audit.updated_by,
        });
        Ok(saved)
    }

    async fn find_by_id(&self, conn: &mut PgConnection, id: TSID) -> Result<Client, BusinessError> {
        let id_raw = id.number() as i64;
        let mut query = QueryBuilder::<sqlx::Postgres>::new(CLIENT_SELECT);
        let row = query
            .push(" AND c.id = ")
            .push_bind(id_raw)
            .build_query_as::<ClientRow>()
            .fetch_optional(&mut *conn)
            .await
            .map_err(|err| {
                error!("Failed to query client {id}: {err:?}");
                ClientBusinessErrors::query_failed(err)
            })?
            .ok_or_else(|| ClientBusinessErrors::not_found(id))?;
        row.try_into()
    }

    async fn delete(
        &self,
        conn: &mut PgConnection,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError> {
        let result = sqlx::query(
            "UPDATE clients SET deleted = TRUE, updated_at = $2, updated_by = $3 WHERE id = $1 AND deleted = FALSE",
        )
        .bind(id.number() as i64)
        .bind(Utc::now())
        .bind(updated_by)
        .execute(&mut *conn)
        .await
        .map_err(|err| {
            error!("Failed to delete client {id}: {err:?}");
            ClientBusinessErrors::delete_failed(err)
        })?;

        if result.rows_affected() == 0 {
            return Err(ClientBusinessErrors::not_found(id));
        }
        Ok(())
    }

    async fn find_page(
        &self,
        conn: &mut PgConnection,
        name: Option<&str>,
        organization_id: Option<TSID>,
        page_request: &PageRequest,
    ) -> Result<Page<Client>, BusinessError> {
        let name = name.map(str::trim).filter(|value| !value.is_empty());
        let mut query = QueryBuilder::<sqlx::Postgres>::new(CLIENT_SELECT);
        if let Some(name) = name {
            query
                .push(" AND c.name ILIKE ")
                .push_bind(format!("%{name}%"));
        }
        if let Some(organization_id) = organization_id {
            query
                .push(" AND c.organization_id = ")
                .push_bind(organization_id.number() as i64);
        }
        let clients = query
            .push(" ORDER BY c.name, c.id LIMIT ")
            .push_bind(page_request.limit() as i64)
            .push(" OFFSET ")
            .push_bind(page_request.offset() as i64)
            .build_query_as::<ClientRow>()
            .fetch_all(&mut *conn)
            .await
            .map_err(|err| {
                error!("Failed to list clients: {err:?}");
                ClientBusinessErrors::query_failed(err)
            })?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Client>, BusinessError>>()?;

        let mut count_query = QueryBuilder::<sqlx::Postgres>::new(
            "SELECT COUNT(*) FROM clients c WHERE c.deleted = FALSE",
        );
        if let Some(name) = name {
            count_query
                .push(" AND c.name ILIKE ")
                .push_bind(format!("%{name}%"));
        }
        if let Some(organization_id) = organization_id {
            count_query
                .push(" AND c.organization_id = ")
                .push_bind(organization_id.number() as i64);
        }
        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(&mut *conn)
            .await
            .map_err(|err| {
                error!("Failed to count clients: {err:?}");
                ClientBusinessErrors::query_failed(err)
            })?;

        Ok(Page::new(
            clients,
            page_request.page,
            page_request.page_size,
            total as u64,
        ))
    }
}
