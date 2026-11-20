use crate::domain::{Product, ProductId, ProductRepository, OrganizationId};
use crate::errors::ProductBusinessErrors;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared::entities::{AuditFields, BusinessError};
use shared::types::{Page, PageRequest};
use sqlx::{PgConnection, QueryBuilder};
use tracing::error;
use tsid::TSID;

const PRODUCT_SELECT: &str = r#"
    SELECT id, organization_id, name, description, sku, price,
           deleted, created_at, updated_at, created_by, updated_by
    FROM products
    WHERE deleted = FALSE
"#;

#[derive(sqlx::FromRow)]
struct ProductRow {
    id: i64,
    organization_id: i64,
    name: String,
    description: Option<String>,
    sku: Option<String>,
    price: Option<f64>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: String,
    updated_by: String,
}

impl TryFrom<ProductRow> for Product {
    type Error = BusinessError;

    fn try_from(row: ProductRow) -> Result<Self, Self::Error> {
        Ok(Product::new(
            ProductId::new(TSID::from(row.id as u64)),
            OrganizationId::new(TSID::from(row.organization_id as u64)),
            row.name,
            row.description,
            row.sku,
            row.price,
        )
        .with_audit(AuditFields {
            created_at: row.created_at,
            updated_at: row.updated_at,
            created_by: row.created_by,
            updated_by: row.updated_by,
        }))
    }
}

#[derive(Debug, Clone, Default)]
pub struct PostgresProductRepository;

impl PostgresProductRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProductRepository for PostgresProductRepository {
    async fn save(
        &self,
        conn: &mut PgConnection,
        product: &Product,
    ) -> Result<Product, BusinessError> {
        let now = Utc::now();
        let audit = product
            .audit()
            .cloned()
            .unwrap_or_else(|| AuditFields::created("system"));

        sqlx::query(
            r#"
                INSERT INTO products (
                    id, organization_id, name, description, sku, price,
                    deleted, created_at, updated_at, created_by, updated_by
                ) VALUES ($1, $2, $3, $4, $5, $6, FALSE, $7, $8, $9, $10)
                ON CONFLICT (id) DO UPDATE SET
                    organization_id = EXCLUDED.organization_id,
                    name = EXCLUDED.name,
                    description = EXCLUDED.description,
                    sku = EXCLUDED.sku,
                    price = EXCLUDED.price,
                    deleted = FALSE,
                    updated_at = EXCLUDED.updated_at,
                    updated_by = EXCLUDED.updated_by
            "#,
        )
        .bind(product.id().number() as i64)
        .bind(product.organization_id().number() as i64)
        .bind(product.name())
        .bind(product.description().map(String::from))
        .bind(product.sku().map(String::from))
        .bind(product.price())
        .bind(audit.created_at)
        .bind(now.max(audit.updated_at))
        .bind(&audit.created_by)
        .bind(&audit.updated_by)
        .execute(&mut *conn)
        .await
        .map_err(|err| {
            if let Some(db_error) = err.as_database_error() {
                if db_error.code().as_deref() == Some("23503") {
                    return ProductBusinessErrors::organization_not_found();
                }
            }
            error!("Failed to persist product {}: {err:?}", product.id());
            ProductBusinessErrors::persist_failed(err)
        })?;

        let saved = product.clone().with_audit(AuditFields {
            created_at: audit.created_at,
            updated_at: now.max(audit.updated_at),
            created_by: audit.created_by,
            updated_by: audit.updated_by,
        });
        Ok(saved)
    }

    async fn find_by_id(&self, conn: &mut PgConnection, id: TSID) -> Result<Product, BusinessError> {
        let id_raw = id.number() as i64;
        let mut query = QueryBuilder::<sqlx::Postgres>::new(PRODUCT_SELECT);
        let row = query
            .push(" AND id = ")
            .push_bind(id_raw)
            .build_query_as::<ProductRow>()
            .fetch_optional(&mut *conn)
            .await
            .map_err(|err| {
                error!("Failed to query product {id}: {err:?}");
                ProductBusinessErrors::query_failed(err)
            })?
            .ok_or_else(|| ProductBusinessErrors::not_found(id))?;
        row.try_into()
    }

    async fn delete(
        &self,
        conn: &mut PgConnection,
        id: TSID,
        updated_by: &str,
    ) -> Result<(), BusinessError> {
        let result = sqlx::query(
            "UPDATE products SET deleted = TRUE, updated_at = $2, updated_by = $3 WHERE id = $1 AND deleted = FALSE",
        )
        .bind(id.number() as i64)
        .bind(Utc::now())
        .bind(updated_by)
        .execute(&mut *conn)
        .await
        .map_err(|err| {
            error!("Failed to delete product {id}: {err:?}");
            ProductBusinessErrors::delete_failed(err)
        })?;

        if result.rows_affected() == 0 {
            return Err(ProductBusinessErrors::not_found(id));
        }
        Ok(())
    }

    async fn find_page(
        &self,
        conn: &mut PgConnection,
        name: Option<&str>,
        organization_id: Option<TSID>,
        page_request: &PageRequest,
    ) -> Result<Page<Product>, BusinessError> {
        let name = name.map(str::trim).filter(|value| !value.is_empty());
        let mut query = QueryBuilder::<sqlx::Postgres>::new(PRODUCT_SELECT);
        if let Some(name) = name {
            query
                .push(" AND name ILIKE ")
                .push_bind(format!("%{name}%"));
        }
        if let Some(organization_id) = organization_id {
            query
                .push(" AND organization_id = ")
                .push_bind(organization_id.number() as i64);
        }
        let products = query
            .push(" ORDER BY name, id LIMIT ")
            .push_bind(page_request.limit() as i64)
            .push(" OFFSET ")
            .push_bind(page_request.offset() as i64)
            .build_query_as::<ProductRow>()
            .fetch_all(&mut *conn)
            .await
            .map_err(|err| {
                error!("Failed to list products: {err:?}");
                ProductBusinessErrors::query_failed(err)
            })?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Product>, BusinessError>>()?;

        let mut count_query = QueryBuilder::<sqlx::Postgres>::new(
            "SELECT COUNT(*) FROM products WHERE deleted = FALSE",
        );
        if let Some(name) = name {
            count_query
                .push(" AND name ILIKE ")
                .push_bind(format!("%{name}%"));
        }
        if let Some(organization_id) = organization_id {
            count_query
                .push(" AND organization_id = ")
                .push_bind(organization_id.number() as i64);
        }
        let total: i64 = count_query
            .build_query_scalar()
            .fetch_one(&mut *conn)
            .await
            .map_err(|err| {
                error!("Failed to count products: {err:?}");
                ProductBusinessErrors::query_failed(err)
            })?;

        Ok(Page::new(
            products,
            page_request.page,
            page_request.page_size,
            total as u64,
        ))
    }
}
