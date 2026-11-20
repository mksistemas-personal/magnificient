use crate::application::ProductPersistence;
use crate::application::delete_organization_products_usecase::data::DeleteOrganizationProductsRequest;
use shared::entities::BusinessError;
use shared::types::PageRequest;

#[derive(Clone)]
pub struct DeleteOrganizationProductsService {
    persistence: ProductPersistence,
}

impl DeleteOrganizationProductsService {
    pub fn new(persistence: ProductPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(
        &self,
        request: DeleteOrganizationProductsRequest,
    ) -> Result<(), BusinessError> {
        loop {
            let page = self
                .persistence
                .find_page(
                    None,
                    Some(request.organization_id.value()),
                    PageRequest::new(1, 20),
                )
                .await?;
            if page.content.is_empty() {
                return Ok(());
            }

            for product in page.content {
                self.persistence
                    .delete(product.id().value(), request.updated_by.clone())
                    .await?;
            }
        }
    }
}
