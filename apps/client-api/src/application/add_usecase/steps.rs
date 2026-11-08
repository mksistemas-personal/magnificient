use crate::application::ClientBusinessErrors;
use crate::application::ClientPersistence;
use crate::application::add_usecase::data::{
    AddressValidatedContextData, BuiltContextData, InitialContextData, ValidatedContextData,
};
use crate::application::add_usecase::states::{AddressValidated, Built, ClientValidated, Initial};
use crate::domain::{Client, ClientId};
use async_trait::async_trait;
use shared::entities::use_case::context::Context;
use shared::entities::{AuditFields, BusinessError};
use tsid::create_tsid;

pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<ClientValidated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<ClientValidated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        if request.name.trim().is_empty() {
            return Err(ClientBusinessErrors::name_empty());
        }
        if request.created_by.trim().is_empty() {
            return Err(ClientBusinessErrors::creator_required());
        }
        if !request.organization_id.is_valid() {
            return Err(ClientBusinessErrors::organization_required());
        }
        Ok(Context::new(ValidatedContextData { request }))
    }
}

pub trait ValidateAddressContext {
    fn validate_address(
        self,
    ) -> Result<Context<AddressValidated, AddressValidatedContextData>, BusinessError>;
}

impl ValidateAddressContext for Context<ClientValidated, ValidatedContextData> {
    fn validate_address(
        self,
    ) -> Result<Context<AddressValidated, AddressValidatedContextData>, BusinessError> {
        if let Some(address) = &self.data.request.second_address {
            address.validate()?;
        }
        Ok(Context::new(AddressValidatedContextData {
            organization_id: self.data.request.organization_id,
            request: self.data.request,
        }))
    }
}

pub trait BuildClientContext {
    fn build(self) -> Result<Context<Built, BuiltContextData>, BusinessError>;
}

impl BuildClientContext for Context<AddressValidated, AddressValidatedContextData> {
    fn build(self) -> Result<Context<Built, BuiltContextData>, BusinessError> {
        let request = self.data.request;
        let client = Client::new(
            ClientId::new(create_tsid()),
            self.data.organization_id,
            request.name.trim(),
            normalize_nickname(request.nickname),
            request.second_address,
        )
        .with_audit(AuditFields::created(request.created_by.trim()));
        Ok(Context::new(BuiltContextData { client }))
    }
}

#[async_trait]
pub trait PersistClientContext {
    async fn save(self, persistence: &ClientPersistence) -> Result<Client, BusinessError>;
}

#[async_trait]
impl PersistClientContext for Context<Built, BuiltContextData> {
    async fn save(self, persistence: &ClientPersistence) -> Result<Client, BusinessError> {
        persistence.save(self.data.client).await
    }
}

fn normalize_nickname(nickname: Option<String>) -> Option<String> {
    nickname
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
