use crate::application::ProductBusinessErrors;
use crate::application::ProductPersistence;
use crate::application::add_usecase::data::{BuiltContextData, InitialContextData, ValidatedContextData};
use crate::application::add_usecase::states::{Built, ProductValidated, Initial};
use crate::domain::{Product, ProductId};
use async_trait::async_trait;
use shared::entities::{AuditFields, BusinessError, Context};
use tsid::create_tsid;

pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<ProductValidated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<ProductValidated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        if request.name.trim().is_empty() {
            return Err(ProductBusinessErrors::name_empty());
        }
        if request.created_by.trim().is_empty() {
            return Err(ProductBusinessErrors::creator_required());
        }
        if !request.organization_id.is_valid() {
            return Err(ProductBusinessErrors::organization_required());
        }
        Ok(Context::new(ValidatedContextData { request }))
    }
}

pub trait BuildProductContext {
    fn build(self) -> Result<Context<Built, BuiltContextData>, BusinessError>;
}

impl BuildProductContext for Context<ProductValidated, ValidatedContextData> {
    fn build(self) -> Result<Context<Built, BuiltContextData>, BusinessError> {
        let request = self.data.request;
        let product = Product::new(
            ProductId::new(create_tsid()),
            request.organization_id,
            request.name.trim(),
            request.description.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty()),
            request.sku.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty()),
            request.price,
        )
        .with_audit(AuditFields::created(request.created_by.trim()));
        Ok(Context::new(BuiltContextData { product }))
    }
}

#[async_trait]
pub trait PersistProductContext {
    async fn save(self, persistence: &ProductPersistence) -> Result<Product, BusinessError>;
}

#[async_trait]
impl PersistProductContext for Context<Built, BuiltContextData> {
    async fn save(self, persistence: &ProductPersistence) -> Result<Product, BusinessError> {
        persistence.save(self.data.product).await
    }
}
