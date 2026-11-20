use crate::application::SupplierBusinessErrors;
use crate::application::SupplierPersistence;
use crate::application::add_usecase::data::{
    AddressValidatedContextData, BuiltContextData, InitialContextData, ValidatedContextData,
};
use crate::application::add_usecase::states::{AddressValidated, Built, SupplierValidated, Initial};
use crate::domain::{Supplier, SupplierId};
use async_trait::async_trait;
use shared::entities::{AuditFields, BusinessError, Context};
use tsid::create_tsid;

pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<SupplierValidated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<SupplierValidated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        if request.name.trim().is_empty() {
            return Err(SupplierBusinessErrors::name_empty());
        }
        if request.created_by.trim().is_empty() {
            return Err(SupplierBusinessErrors::creator_required());
        }
        if !request.organization_id.is_valid() {
            return Err(SupplierBusinessErrors::organization_required());
        }
        Ok(Context::new(ValidatedContextData { request }))
    }
}

pub trait ValidateAddressContext {
    fn validate_address(
        self,
    ) -> Result<Context<AddressValidated, AddressValidatedContextData>, BusinessError>;
}

impl ValidateAddressContext for Context<SupplierValidated, ValidatedContextData> {
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

pub trait BuildSupplierContext {
    fn build(self) -> Result<Context<Built, BuiltContextData>, BusinessError>;
}

impl BuildSupplierContext for Context<AddressValidated, AddressValidatedContextData> {
    fn build(self) -> Result<Context<Built, BuiltContextData>, BusinessError> {
        let request = self.data.request;
        let supplier = Supplier::new(
            SupplierId::new(create_tsid()),
            self.data.organization_id,
            request.name.trim(),
            normalize_nickname(request.nickname),
            request.second_address,
        )
        .with_audit(AuditFields::created(request.created_by.trim()));
        Ok(Context::new(BuiltContextData { supplier }))
    }
}

#[async_trait]
pub trait PersistSupplierContext {
    async fn save(self, persistence: &SupplierPersistence) -> Result<Supplier, BusinessError>;
}

#[async_trait]
impl PersistSupplierContext for Context<Built, BuiltContextData> {
    async fn save(self, persistence: &SupplierPersistence) -> Result<Supplier, BusinessError> {
        persistence.save(self.data.supplier).await
    }
}

fn normalize_nickname(nickname: Option<String>) -> Option<String> {
    nickname
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
