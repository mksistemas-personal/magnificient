use crate::domain::{Organization, OrganizationRepository};
use database::UnitOfWork;
use shared::entities::{AddressRepository, BusinessError};
use shared::types::PageRequest;
use shared::types::page::Page;
use sqlx::{Postgres, Transaction};
use std::sync::Arc;
use tsid::TSID;

/// Persistence dependencies and transaction policy shared by organization use cases.
#[derive(Clone)]
pub struct OrganizationPersistence {
    pub(crate) repository: Arc<dyn OrganizationRepository>,
    pub(crate) address_repository: Option<Arc<dyn AddressRepository>>,
    pub(crate) unit_of_work: Option<UnitOfWork>,
}

impl OrganizationPersistence {
    pub fn new(repository: impl OrganizationRepository + 'static) -> Self {
        Self {
            repository: Arc::new(repository),
            address_repository: None,
            unit_of_work: None,
        }
    }

    pub fn with_address_repository(
        mut self,
        address_repository: impl AddressRepository + 'static,
    ) -> Self {
        self.address_repository = Some(Arc::new(address_repository));
        self
    }

    pub fn with_unit_of_work(mut self, unit_of_work: UnitOfWork) -> Self {
        self.unit_of_work = Some(unit_of_work);
        self
    }

    pub async fn save(&self, organization: Organization) -> Result<Organization, BusinessError> {
        match self.unit_of_work.as_ref() {
            Some(unit_of_work) => {
                let repository = Arc::clone(&self.repository);
                let address_repository = self.address_repository.clone();
                unit_of_work
                    .transaction(move |tx| {
                        Box::pin(async move {
                            Self::save_in_transaction(
                                repository.as_ref(),
                                address_repository.as_deref(),
                                tx,
                                organization,
                            )
                            .await
                        })
                    })
                    .await
            }
            None => {
                Self::save_without_transaction(
                    self.repository.as_ref(),
                    self.address_repository.as_deref(),
                    organization,
                )
                .await
            }
        }
    }

    pub async fn find_by_id(&self, id: TSID) -> Result<Organization, BusinessError> {
        match self.unit_of_work.as_ref() {
            Some(unit_of_work) => {
                let repository = Arc::clone(&self.repository);
                unit_of_work
                    .transaction(move |tx| {
                        Box::pin(async move { repository.find_by_id_in_tx(tx, id).await })
                    })
                    .await
            }
            None => self.repository.find_by_id(id).await,
        }
    }

    pub async fn delete(&self, id: TSID, updated_by: &str) -> Result<(), BusinessError> {
        match self.unit_of_work.as_ref() {
            Some(unit_of_work) => {
                let repository = Arc::clone(&self.repository);
                let updated_by = updated_by.to_owned();
                unit_of_work
                    .transaction(move |tx| {
                        Box::pin(async move { repository.delete_in_tx(tx, id, &updated_by).await })
                    })
                    .await
            }
            None => self.repository.delete(id, updated_by).await,
        }
    }

    pub async fn find_page(
        &self,
        name: Option<&str>,
        page_request: PageRequest,
    ) -> Result<Page<Organization>, BusinessError> {
        match self.unit_of_work.as_ref() {
            Some(unit_of_work) => {
                let repository = Arc::clone(&self.repository);
                let name = name.map(str::to_owned);
                unit_of_work
                    .transaction(move |tx| {
                        Box::pin(async move {
                            let content = repository
                                .find_all_in_tx(&mut *tx, name.as_deref(), &page_request)
                                .await?;
                            let total = repository.count_in_tx(&mut *tx, name.as_deref()).await?;
                            Ok(Page::new(
                                content,
                                page_request.page,
                                page_request.page_size,
                                total,
                            ))
                        })
                    })
                    .await
            }
            None => {
                let content = self.repository.find_all(name, &page_request).await?;
                let total = self.repository.count(name).await?;
                Ok(Page::new(
                    content,
                    page_request.page,
                    page_request.page_size,
                    total,
                ))
            }
        }
    }

    pub async fn find_by_id_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: TSID,
    ) -> Result<Organization, BusinessError> {
        self.repository.find_by_id_in_tx(tx, id).await
    }

    pub async fn save_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        organization: Organization,
    ) -> Result<Organization, BusinessError> {
        Self::save_in_transaction(
            self.repository.as_ref(),
            self.address_repository.as_deref(),
            tx,
            organization,
        )
        .await
    }

    async fn save_without_transaction(
        repository: &dyn OrganizationRepository,
        address_repository: Option<&dyn AddressRepository>,
        mut organization: Organization,
    ) -> Result<Organization, BusinessError> {
        if let (Some(address_repository), Some(address)) =
            (address_repository, organization.address().cloned())
        {
            organization = organization.with_address(address_repository.save(&address).await?);
        }
        repository.save(&organization).await
    }

    async fn save_in_transaction(
        repository: &dyn OrganizationRepository,
        address_repository: Option<&dyn AddressRepository>,
        tx: &mut Transaction<'_, Postgres>,
        mut organization: Organization,
    ) -> Result<Organization, BusinessError> {
        if let (Some(address_repository), Some(address)) =
            (address_repository, organization.address().cloned())
        {
            organization =
                organization.with_address(address_repository.save_in_tx(tx, &address).await?);
        }
        repository.save_in_tx(tx, &organization).await
    }
}
