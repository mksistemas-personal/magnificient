use crate::application::OrganizationBusinessErrors;
use crate::application::addusecase::data::{
    AddressValidatedContextData, CreatedContextData, InitialContextData, ValidatedContextData,
};
use crate::application::addusecase::{AddressValidated, Built, Initial, OrganizationValidated};
use crate::domain::{Organization, OrganizationId, OrganizationRepository};
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::use_case::context::Context;
use shared::entities::{AddressRepository, AuditFields, BusinessError, Document};
use tsid::create_tsid;

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
        let organization_id = OrganizationId::new(create_tsid());

        let name = request.name.clone();
        if name.trim().is_empty() {
            return Err(OrganizationBusinessErrors::name_empty());
        }

        let document =
            Document::new_validated(&request.document_value, request.document_type.clone())
                .map_err(|_| OrganizationBusinessErrors::document_invalid())?;

        let created_by = request.created_by.clone();
        if created_by.trim().is_empty() {
            return Err(OrganizationBusinessErrors::creator_required());
        }

        let audit = AuditFields::created(&request.created_by);

        Ok(Context::<OrganizationValidated, ValidatedContextData>::new(
            ValidatedContextData {
                request,
                organization_id,
                document,
                audit,
            },
        ))
    }
}

pub trait ValidateAddressContext {
    fn validate_address(
        self,
    ) -> Result<Context<AddressValidated, AddressValidatedContextData>, BusinessError>;
}

impl ValidateAddressContext for Context<OrganizationValidated, ValidatedContextData> {
    fn validate_address(
        self,
    ) -> Result<Context<AddressValidated, AddressValidatedContextData>, BusinessError> {
        let request = self.data.request;
        let organization_id = self.data.organization_id;
        let document = self.data.document;
        let audit = self.data.audit;

        let address = if let Some(addr) = request.address.clone() {
            addr.validate()?;
            Some(addr)
        } else {
            None
        };

        Ok(
            Context::<AddressValidated, AddressValidatedContextData>::new(
                AddressValidatedContextData {
                    request,
                    organization_id,
                    document,
                    address,
                    audit,
                },
            ),
        )
    }
}

pub trait BuildOrganizationContext {
    fn build(self) -> Result<Context<Built, CreatedContextData>, BusinessError>;
}

impl BuildOrganizationContext for Context<AddressValidated, AddressValidatedContextData> {
    fn build(self) -> Result<Context<Built, CreatedContextData>, BusinessError> {
        let request = self.data.request;
        let mut organization = Organization::new(
            self.data.organization_id,
            request.name.clone(),
            self.data.document,
        )
        .with_audit(self.data.audit);

        if let Some(address) = self.data.address {
            organization = organization.with_address(address);
        }

        Ok(Context::<Built, CreatedContextData>::new(
            CreatedContextData {
                request,
                organization,
            },
        ))
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
impl PersistOrganizationContext for Context<Built, CreatedContextData> {
    async fn save(
        self,
        repository: &dyn OrganizationRepository,
        address_repository: Option<&dyn AddressRepository>,
        uow: Option<&UnitOfWork>,
    ) -> Result<Organization, BusinessError> {
        let organization = self.data.organization;

        UnitOfWork::execute_maybe_tx(uow, || async {
            let mut org = organization.clone();
            if let (Some(address_repo), Some(address)) =
                (address_repository, org.address().cloned())
            {
                let saved_address = address_repo.save(&address).await?;
                org = org.with_address(saved_address);
            }
            repository.save(&org).await
        })
        .await
    }
}
