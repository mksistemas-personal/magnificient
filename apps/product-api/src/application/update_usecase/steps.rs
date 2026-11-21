use crate::application::ProductBusinessErrors;
use crate::application::ProductPersistence;
use crate::application::update_usecase::data::{
    ChangedContextData, InitialContextData, LoadedContextData, ValidatedContextData,
};
use crate::application::update_usecase::states::{
    Changed, ProductValidated, Initial, Loaded,
};
use crate::domain::Product;
use async_trait::async_trait;
use shared::entities::{BusinessError, Context};

pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<ProductValidated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<ProductValidated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        if !request.id.is_valid() {
            return Err(ProductBusinessErrors::not_found(request.id.value()));
        }
        if !request.organization_id.is_valid() {
            return Err(ProductBusinessErrors::organization_required());
        }
        if request.name.trim().is_empty() {
            return Err(ProductBusinessErrors::name_empty());
        }
        if request.updated_by.trim().is_empty() {
            return Err(ProductBusinessErrors::updater_required());
        }
        Ok(Context::new(ValidatedContextData { request }))
    }
}

#[async_trait]
pub trait LoadProductContext {
    async fn load(
        self,
        persistence: &ProductPersistence,
    ) -> Result<Context<Loaded, LoadedContextData>, BusinessError>;
}

#[async_trait]
impl LoadProductContext for Context<ProductValidated, ValidatedContextData> {
    async fn load(
        self,
        persistence: &ProductPersistence,
    ) -> Result<Context<Loaded, LoadedContextData>, BusinessError> {
        let product = persistence.find_by_id(self.data.request.id.value()).await?;
        Ok(Context::new(LoadedContextData {
            request: self.data.request,
            product,
        }))
    }
}

pub trait ChangeProductContext {
    fn change(self) -> Result<Context<Changed, ChangedContextData>, BusinessError>;
}

impl ChangeProductContext for Context<Loaded, LoadedContextData> {
    fn change(mut self) -> Result<Context<Changed, ChangedContextData>, BusinessError> {
        let request = self.data.request;
        self.data.product.update(
            request.organization_id,
            request.name.trim(),
            request.description.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty()),
            request.sku.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty()),
            request.price,
            request.updated_by.trim(),
        );
        Ok(Context::new(ChangedContextData {
            product: self.data.product,
        }))
    }
}

#[async_trait]
pub trait PersistProductContext {
    async fn save(self, persistence: &ProductPersistence) -> Result<Product, BusinessError>;
}

#[async_trait]
impl PersistProductContext for Context<Changed, ChangedContextData> {
    async fn save(self, persistence: &ProductPersistence) -> Result<Product, BusinessError> {
        persistence.save(self.data.product).await
    }
}
