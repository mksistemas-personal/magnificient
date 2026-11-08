use crate::application::ClientPersistence;
use crate::domain::{Client, ClientId, OrganizationId};
use crate::errors::ClientBusinessErrors;
use async_trait::async_trait;
use shared::entities::{
    Address, AsyncAddEntityUseCase, AsyncDeleteEntityUseCase, AsyncGetAllUseCase,
    AsyncGetByIdUseCase, AsyncUpdateEntityUseCase, AuditFields, BusinessError,
};
use shared::types::{Page, PageRequest};
use tsid::create_tsid;

#[derive(Debug, Clone)]
pub struct CreateClientRequest {
    pub organization_id: OrganizationId,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    pub created_by: String,
}

#[derive(Debug, Clone)]
pub struct UpdateClientRequest {
    pub id: ClientId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    pub updated_by: String,
}

#[derive(Debug, Clone)]
pub struct DeleteClientRequest {
    pub id: ClientId,
    pub updated_by: String,
}

#[derive(Debug, Clone, Copy)]
pub struct GetClientByIdRequest {
    pub id: ClientId,
}

#[derive(Debug, Clone)]
pub struct FindClientsRequest {
    pub name: Option<String>,
    pub organization_id: Option<OrganizationId>,
    pub page_request: PageRequest,
}

#[derive(Clone)]
pub struct CreateClientService {
    persistence: ClientPersistence,
}

impl CreateClientService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(&self, request: CreateClientRequest) -> Result<Client, BusinessError> {
        if request.name.trim().is_empty() {
            return Err(ClientBusinessErrors::name_empty());
        }
        if request.created_by.trim().is_empty() {
            return Err(ClientBusinessErrors::creator_required());
        }
        validate_organization_id(request.organization_id)?;
        if let Some(address) = &request.second_address {
            address.validate()?;
        }
        let client = Client::new(
            ClientId::new(create_tsid()),
            request.organization_id,
            request.name.trim(),
            normalize_nickname(request.nickname),
            request.second_address,
        )
        .with_audit(AuditFields::created(request.created_by.trim()));
        self.persistence.save(client).await
    }
}

#[async_trait]
impl AsyncAddEntityUseCase<CreateClientRequest, Client> for CreateClientService {
    async fn execute(&self, request: CreateClientRequest) -> Result<Client, BusinessError> {
        CreateClientService::execute(self, request).await
    }
}

#[derive(Clone)]
pub struct UpdateClientService {
    persistence: ClientPersistence,
}

impl UpdateClientService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(&self, request: UpdateClientRequest) -> Result<Client, BusinessError> {
        if !request.id.is_valid() {
            return Err(ClientBusinessErrors::not_found(request.id.value()));
        }
        if request.name.trim().is_empty() {
            return Err(ClientBusinessErrors::name_empty());
        }
        if request.updated_by.trim().is_empty() {
            return Err(ClientBusinessErrors::updater_required());
        }
        validate_organization_id(request.organization_id)?;
        if let Some(address) = &request.second_address {
            address.validate()?;
        }

        let mut client = self.persistence.find_by_id(request.id.value()).await?;
        let mut second_address = request.second_address;
        if let Some(address) = second_address.as_mut() {
            if address.id.is_none() {
                address.id = client.second_address.as_ref().and_then(Address::id);
            }
            if let Some(audit) = address.audit.as_mut() {
                audit.updated(request.updated_by.trim());
            } else if let Some(mut audit) =
                client.second_address.as_ref().and_then(|a| a.audit.clone())
            {
                audit.updated(request.updated_by.trim());
                address.audit = Some(audit);
            } else {
                address.audit = Some(AuditFields::created(request.updated_by.trim()));
            }
        }
        client.update(
            request.organization_id,
            request.name.trim(),
            normalize_nickname(request.nickname),
            second_address,
            request.updated_by.trim(),
        );
        self.persistence.save(client).await
    }
}

#[async_trait]
impl AsyncUpdateEntityUseCase<UpdateClientRequest, Client> for UpdateClientService {
    async fn execute(&self, request: UpdateClientRequest) -> Result<Client, BusinessError> {
        UpdateClientService::execute(self, request).await
    }
}

#[derive(Clone)]
pub struct DeleteClientService {
    persistence: ClientPersistence,
}

impl DeleteClientService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(&self, request: DeleteClientRequest) -> Result<(), BusinessError> {
        if !request.id.is_valid() {
            return Err(ClientBusinessErrors::not_found(request.id.value()));
        }
        if request.updated_by.trim().is_empty() {
            return Err(ClientBusinessErrors::updater_required());
        }
        self.persistence
            .delete(request.id.value(), request.updated_by.trim().to_owned())
            .await
    }
}

#[async_trait]
impl AsyncDeleteEntityUseCase<DeleteClientRequest, ()> for DeleteClientService {
    async fn execute(&self, request: DeleteClientRequest) -> Result<(), BusinessError> {
        DeleteClientService::execute(self, request).await
    }
}

#[derive(Clone)]
pub struct GetClientByIdService {
    persistence: ClientPersistence,
}

impl GetClientByIdService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(&self, request: GetClientByIdRequest) -> Result<Client, BusinessError> {
        if !request.id.is_valid() {
            return Err(ClientBusinessErrors::not_found(request.id.value()));
        }
        self.persistence.find_by_id(request.id.value()).await
    }
}

#[async_trait]
impl AsyncGetByIdUseCase<GetClientByIdRequest, Client> for GetClientByIdService {
    async fn execute(&self, request: GetClientByIdRequest) -> Result<Client, BusinessError> {
        GetClientByIdService::execute(self, request).await
    }
}

#[derive(Clone)]
pub struct FindClientsService {
    persistence: ClientPersistence,
}

impl FindClientsService {
    pub fn new(persistence: ClientPersistence) -> Self {
        Self { persistence }
    }

    pub async fn execute(
        &self,
        request: FindClientsRequest,
    ) -> Result<Page<Client>, BusinessError> {
        if request.page_request.page == 0 || request.page_request.page_size == 0 {
            return Err(ClientBusinessErrors::pagination_invalid());
        }
        if request
            .organization_id
            .is_some_and(|organization_id| !organization_id.is_valid())
        {
            return Err(ClientBusinessErrors::organization_required());
        }
        self.persistence
            .find_page(
                request.name,
                request.organization_id.map(|id| id.value()),
                request.page_request,
            )
            .await
    }
}

#[async_trait]
impl AsyncGetAllUseCase<FindClientsRequest, Page<Client>> for FindClientsService {
    async fn execute(&self, request: FindClientsRequest) -> Result<Page<Client>, BusinessError> {
        FindClientsService::execute(self, request).await
    }
}

fn validate_organization_id(id: OrganizationId) -> Result<(), BusinessError> {
    if id.is_valid() {
        Ok(())
    } else {
        Err(ClientBusinessErrors::organization_required())
    }
}

fn normalize_nickname(nickname: Option<String>) -> Option<String> {
    nickname
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::InMemoryClientRepository;
    use shared::entities::{Address, InMemoryAddressRepository};

    fn persistence() -> ClientPersistence {
        ClientPersistence::new_without_pool(
            InMemoryClientRepository::new(),
            InMemoryAddressRepository::new(),
        )
    }

    fn create_request() -> CreateClientRequest {
        CreateClientRequest {
            organization_id: OrganizationId::new(tsid::TSID::from(500u64)),
            name: "Morgan Lee".to_owned(),
            nickname: Some("  Mo  ".to_owned()),
            second_address: Some(Address::new(
                "10 Main Street",
                "10",
                "",
                "Downtown",
                "Springfield",
                "California",
                "CA",
                "90210",
                "United States",
                "US",
                "",
            )),
            created_by: "api-user".to_owned(),
        }
    }

    #[tokio::test]
    async fn client_crud_flow_uses_typed_ids_and_audit() {
        let persistence = persistence();
        let created = CreateClientService::new(persistence.clone())
            .execute(create_request())
            .await
            .unwrap();
        assert_eq!(created.nickname.as_deref(), Some("Mo"));
        assert!(created.second_address.as_ref().unwrap().id().is_some());
        assert!(created.audit.is_some());

        let updated = UpdateClientService::new(persistence.clone())
            .execute(UpdateClientRequest {
                id: created.id,
                organization_id: created.organization_id,
                name: "Morgan Lee Updated".to_owned(),
                nickname: None,
                second_address: None,
                updated_by: "editor".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(updated.name, "Morgan Lee Updated");
        assert_eq!(updated.audit.as_ref().unwrap().updated_by(), "editor");

        let page = FindClientsService::new(persistence.clone())
            .execute(FindClientsRequest {
                name: Some("Morgan".to_owned()),
                organization_id: Some(created.organization_id),
                page_request: PageRequest::default(),
            })
            .await
            .unwrap();
        assert_eq!(page.total_elements, 1);

        DeleteClientService::new(persistence.clone())
            .execute(DeleteClientRequest {
                id: created.id,
                updated_by: "deleter".to_owned(),
            })
            .await
            .unwrap();
        let missing = GetClientByIdService::new(persistence)
            .execute(GetClientByIdRequest { id: created.id })
            .await;
        assert!(matches!(missing, Err(BusinessError::NotFound(_))));
    }
}
