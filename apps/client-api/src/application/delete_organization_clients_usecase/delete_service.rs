use crate::application::ClientPersistence;
use crate::application::delete_organization_clients_usecase::data::DeleteOrganizationClientsRequest;
use shared::entities::BusinessError;
use shared::types::PageRequest;

#[derive(Clone)]
pub struct DeleteOrganizationClientsService {
    persistence: ClientPersistence,
}

impl DeleteOrganizationClientsService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(
        &self,
        request: DeleteOrganizationClientsRequest,
    ) -> Result<(), BusinessError> {
        // Always read the first page: deleting each batch changes the result set,
        // so advancing the offset could skip clients.
        loop {
            let page = self
                .persistence
                .find_page(
                    None,
                    Some(request.organization_id.value()),
                    PageRequest::new(1, 100),
                )
                .await?;
            if page.content.is_empty() {
                return Ok(());
            }

            for client in page.content {
                self.persistence
                    .delete(client.id.value(), request.updated_by.clone())
                    .await?;
            }
        }
    }
}
