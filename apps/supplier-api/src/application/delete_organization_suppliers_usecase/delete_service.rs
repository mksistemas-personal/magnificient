use crate::application::SupplierPersistence;
use crate::application::delete_organization_suppliers_usecase::data::DeleteOrganizationSuppliersRequest;
use shared::entities::BusinessError;
use shared::types::PageRequest;

#[derive(Clone)]
pub struct DeleteOrganizationSuppliersService {
    persistence: SupplierPersistence,
}

impl DeleteOrganizationSuppliersService {
    pub fn new(persistence: SupplierPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(
        &self,
        request: DeleteOrganizationSuppliersRequest,
    ) -> Result<(), BusinessError> {
        // Always read the first page: deleting each batch changes the result set,
        // so advancing the offset could skip suppliers.
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

            for supplier in page.content {
                self.persistence
                    .delete(supplier.id().value(), request.updated_by.clone())
                    .await?;
            }
        }
    }
}
