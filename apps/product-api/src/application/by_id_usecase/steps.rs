use crate::application::by_id_usecase::data::{InitialContextData, ValidatedContextData};
use crate::application::by_id_usecase::states::{Initial, Validated};
use crate::application::{ProductBusinessErrors, ProductPersistence};
use crate::domain::Product;
use async_trait::async_trait;
use shared::entities::{BusinessError, Context};
pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError>;
}
impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<Validated, ValidatedContextData>, BusinessError> {
        if !self.data.request.id.is_valid() {
            return Err(ProductBusinessErrors::not_found(
                self.data.request.id.value(),
            ));
        }
        Ok(Context::new(ValidatedContextData {
            request: self.data.request,
        }))
    }
}
#[async_trait]
pub trait FindProductByIdContext {
    async fn find(self, persistence: &ProductPersistence) -> Result<Product, BusinessError>;
}
#[async_trait]
impl FindProductByIdContext for Context<Validated, ValidatedContextData> {
    async fn find(self, persistence: &ProductPersistence) -> Result<Product, BusinessError> {
        persistence.find_by_id(self.data.request.id.value()).await
    }
}
