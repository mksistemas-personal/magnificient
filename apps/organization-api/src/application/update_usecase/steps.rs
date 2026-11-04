use crate::application::OrganizationBusinessErrors;
use crate::application::update_usecase::data::{
    AddressValidatedContextData, ChangedContextData, InitialContextData, LoadedContextData,
    ValidatedContextData,
};
use crate::application::update_usecase::states::{
    AddressValidated, Changed, Initial, Loaded, OrganizationValidated, Validated,
};
use crate::domain::{Organization, OrganizationRepository};
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::use_case::context::Context;
use shared::entities::{AddressRepository, AuditFields, BusinessError};

pub trait ValidateInitialContext {
    fn validate(
        self,
    ) -> Result<Context<OrganizationValidated, ValidatedContextData>, BusinessError>;
}

impl ValidateInitialContext for Context<Initial, InitialContextData> {
    fn validate(
        self,
    ) -> Result<Context<OrganizationValidated, ValidatedContextData>, BusinessError> {
        let request = self.data.request;

        if !request.organization_id.is_valid() {
            return Err(OrganizationBusinessErrors::id_invalid());
        }
        if request.name.trim().is_empty() {
            return Err(OrganizationBusinessErrors::name_empty());
        }
        if request.updated_by.trim().is_empty() {
            return Err(OrganizationBusinessErrors::updater_required());
        }

        Ok(Context::new(ValidatedContextData { request }))
    }
}

pub trait ValidateAddressContext {
    fn validate_address(
        self,
    ) -> Result<Context<AddressValidated, AddressValidatedContextData>, BusinessError>;
}

impl ValidateAddressContext for Context<Validated, ValidatedContextData> {
    fn validate_address(
        self,
    ) -> Result<Context<AddressValidated, AddressValidatedContextData>, BusinessError> {
        let request = self.data.request;

        let address = if let Some(addr) = request.address.clone() {
            addr.validate()?;
            Some(addr)
        } else {
            None
        };

        Ok(Context::new(AddressValidatedContextData {
            request,
            address,
        }))
    }
}

#[async_trait]
pub trait LoadOrganizationContext {
    async fn load(
        self,
        repository: &dyn OrganizationRepository,
        uow: Option<&UnitOfWork>,
    ) -> Result<Context<Loaded, LoadedContextData>, BusinessError>;
}

#[async_trait]
impl LoadOrganizationContext for Context<AddressValidated, AddressValidatedContextData> {
    async fn load(
        self,
        repository: &dyn OrganizationRepository,
        uow: Option<&UnitOfWork>,
    ) -> Result<Context<Loaded, LoadedContextData>, BusinessError> {
        let org_id = self.data.request.organization_id;
        let organization = match uow {
            Some(uow) => {
                uow.run_tx(|mut tx| async move {
                    let organization = repository.find_by_id_in_tx(&mut tx, *org_id).await?;
                    Ok((organization, tx))
                })
                .await?
            }
            None => repository.find_by_id(*org_id).await?,
        };

        Ok(Context::new(LoadedContextData {
            request: self.data.request,
            address: self.data.address,
            organization,
        }))
    }
}

pub trait ChangeOrganizationContext {
    fn change(self) -> Result<Context<Changed, ChangedContextData>, BusinessError>;
}

impl ChangeOrganizationContext for Context<Loaded, LoadedContextData> {
    fn change(mut self) -> Result<Context<Changed, ChangedContextData>, BusinessError> {
        let name = self.data.request.name;
        let updated_by = self.data.request.updated_by;

        self.data.organization.update(name, &updated_by);

        if let Some(mut new_address) = self.data.address {
            if new_address.id.is_none() {
                new_address.id = self.data.organization.address().and_then(|a| a.id);
            }
            if let Some(audit) = new_address.audit.as_mut() {
                audit.updated(&updated_by);
            } else if let Some(existing_addr) = self.data.organization.address() {
                if let Some(mut existing_audit) = existing_addr.audit.clone() {
                    existing_audit.updated(&updated_by);
                    new_address.audit = Some(existing_audit);
                } else {
                    new_address.audit = Some(AuditFields::created(&updated_by));
                }
            } else {
                new_address.audit = Some(AuditFields::created(&updated_by));
            }
            self.data.organization.set_address(Some(new_address));
        }

        Ok(Context::new(ChangedContextData {
            organization: self.data.organization,
        }))
    }
}

#[async_trait]
pub trait PersistOrganizationContext {
    async fn save(
        self,
        repository: &dyn OrganizationRepository,
        address_repository: Option<&dyn AddressRepository>,
        uow: Option<&UnitOfWork>,
    ) -> Result<Organization, BusinessError>;
}

#[async_trait]
impl PersistOrganizationContext for Context<Changed, ChangedContextData> {
    async fn save(
        self,
        repository: &dyn OrganizationRepository,
        address_repository: Option<&dyn AddressRepository>,
        uow: Option<&UnitOfWork>,
    ) -> Result<Organization, BusinessError> {
        let organization = self.data.organization;

        match uow {
            Some(uow) => {
                uow.run_tx(|mut tx| async move {
                    let mut org = organization.clone();
                    if let (Some(address_repo), Some(address)) =
                        (address_repository, org.address().cloned())
                    {
                        org = org.with_address(address_repo.save_in_tx(&mut tx, &address).await?);
                    }
                    let saved = repository.save_in_tx(&mut tx, &org).await?;
                    Ok((saved, tx))
                })
                .await
            }
            None => {
                let mut org = organization;
                if let (Some(address_repo), Some(address)) =
                    (address_repository, org.address().cloned())
                {
                    org = org.with_address(address_repo.save(&address).await?);
                }
                repository.save(&org).await
            }
        }
    }
}
