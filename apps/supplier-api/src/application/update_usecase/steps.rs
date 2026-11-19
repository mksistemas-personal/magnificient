use crate::application::SupplierBusinessErrors;
use crate::application::SupplierPersistence;
use crate::application::update_usecase::data::{
    AddressValidatedContextData, ChangedContextData, InitialContextData, LoadedContextData,
    ValidatedContextData,
};
use crate::application::update_usecase::states::{
    AddressValidated, Changed, SupplierValidated, Initial, Loaded,
};
use crate::domain::Supplier;
use async_trait::async_trait;
use shared::entities::{AuditFields, BusinessError, Context};

pub trait ValidateInitialContext {
    fn validate(self) -> Result<Context<SupplierValidated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(self) -> Result<Context<SupplierValidated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;
        if !request.id.is_valid() {
            return Err(SupplierBusinessErrors::not_found(request.id.value()));
        }
        if !request.organization_id.is_valid() {
            return Err(SupplierBusinessErrors::organization_required());
        }
        if request.name.trim().is_empty() {
            return Err(SupplierBusinessErrors::name_empty());
        }
        if request.updated_by.trim().is_empty() {
            return Err(SupplierBusinessErrors::updater_required());
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
            request: self.data.request,
        }))
    }
}

#[async_trait]
pub trait LoadSupplierContext {
    async fn load(
        self,
        persistence: &SupplierPersistence,
    ) -> Result<Context<Loaded, LoadedContextData>, BusinessError>;
}

#[async_trait]
impl LoadSupplierContext for Context<AddressValidated, AddressValidatedContextData> {
    async fn load(
        self,
        persistence: &SupplierPersistence,
    ) -> Result<Context<Loaded, LoadedContextData>, BusinessError> {
        let supplier = persistence.find_by_id(self.data.request.id.value()).await?;
        Ok(Context::new(LoadedContextData {
            request: self.data.request,
            supplier,
        }))
    }
}

pub trait ChangeSupplierContext {
    fn change(self) -> Result<Context<Changed, ChangedContextData>, BusinessError>;
}

impl ChangeSupplierContext for Context<Loaded, LoadedContextData> {
    fn change(mut self) -> Result<Context<Changed, ChangedContextData>, BusinessError> {
        let request = self.data.request;
        let mut second_address = request.second_address;
        if let Some(address) = second_address.as_mut() {
            if address.id.is_none() {
                address.id = self.data.supplier.second_address().and_then(|a| a.id);
            }
            if let Some(audit) = address.audit.as_mut() {
                audit.updated(request.updated_by.trim());
            } else if let Some(mut audit) = self
                .data
                .supplier
                .second_address()
                .and_then(|a| a.audit.clone())
            {
                audit.updated(request.updated_by.trim());
                address.audit = Some(audit);
            } else {
                address.audit = Some(AuditFields::created(request.updated_by.trim()));
            }
        }

        self.data.supplier.update(
            request.organization_id,
            request.name.trim(),
            normalize_nickname(request.nickname),
            second_address,
            request.updated_by.trim(),
        );
        Ok(Context::new(ChangedContextData {
            supplier: self.data.supplier,
        }))
    }
}

#[async_trait]
pub trait PersistSupplierContext {
    async fn save(self, persistence: &SupplierPersistence) -> Result<Supplier, BusinessError>;
}

#[async_trait]
impl PersistSupplierContext for Context<Changed, ChangedContextData> {
    async fn save(self, persistence: &SupplierPersistence) -> Result<Supplier, BusinessError> {
        persistence.save(self.data.supplier).await
    }
}

fn normalize_nickname(nickname: Option<String>) -> Option<String> {
    nickname
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
