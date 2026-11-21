use crate::application::all_usecase::data::{InitialContextData, ValidatedContextData};
use crate::application::all_usecase::states::{Initial, Validated};
use crate::application::{ProductBusinessErrors, ProductPersistence};
use crate::domain::Product;
use async_trait::async_trait;
use shared::entities::{BusinessError, Context};
use shared::types::Page;
pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}
impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        if request.page_request.page == 0 || request.page_request.page_size == 0 {
            return Err(ProductBusinessErrors::pagination_invalid());
        }
        if request.organization_id.is_some_and(|id| !id.is_valid()) {
            return Err(ProductBusinessErrors::organization_required());
        }
        Ok(Context::new(ValidatedContextData { request }))
    }
}
#[async_trait]
pub trait FindProductsContext {
    async fn find(self, persistence: &ProductPersistence) -> Result<Page<Product>, BusinessError>;
}
#[async_trait]
impl FindProductsContext for Context<Validated, ValidatedContextData> {
    async fn find(self, persistence: &ProductPersistence) -> Result<Page<Product>, BusinessError> {
        persistence
            .find_page(
                self.data.request.name,
                self.data.request.organization_id.map(|id| id.value()),
                self.data.request.page_request,
            )
            .await
    }
}
