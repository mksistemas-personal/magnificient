use crate::application::all_usecase::data::*;
use crate::application::all_usecase::states::*;
use crate::application::all_usecase::steps::{FindOrganizationsContext, ValidateInitialContext};
use crate::domain::{Organization, OrganizationRepository};
use async_trait::async_trait;
use database::UnitOfWork;
use shared::entities::BusinessError;
use shared::entities::use_case::context::Context;
use shared::entities::use_case::use_case_traits::AsyncGetAllUseCase;
use shared::types::page::Page;
use std::sync::Arc;

#[derive(Clone)]
pub struct FindAllOrganizationsService {
    repository: Arc<dyn OrganizationRepository>,
    uow: Option<UnitOfWork>,
}

impl FindAllOrganizationsService {
    pub fn new(repository: impl OrganizationRepository + 'static) -> Self {
        Self {
            repository: Arc::new(repository),
            uow: None,
        }
    }

    pub fn new_with_uow(
        repository: impl OrganizationRepository + 'static,
        uow: UnitOfWork,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            uow: Some(uow),
        }
    }

    pub async fn execute(
        &self,
        request: FindAllOrganizationsRequest,
    ) -> Result<Page<Organization>, BusinessError> {
        <Self as AsyncGetAllUseCase<FindAllOrganizationsRequest, Page<Organization>>>::execute(
            self, request,
        )
        .await
    }
}

pub type GetAllOrganizationsService = FindAllOrganizationsService;
pub type FindOrganizationsByNameService = FindAllOrganizationsService;

#[async_trait]
impl AsyncGetAllUseCase<FindAllOrganizationsRequest, Page<Organization>>
    for FindAllOrganizationsService
{
    async fn execute(
        &self,
        request: FindAllOrganizationsRequest,
    ) -> Result<Page<Organization>, BusinessError> {
        Context::<Initial, _>::new(InitialContextData { request })
            .validate()?
            .find(self.repository.as_ref(), self.uow.as_ref())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Organization, OrganizationId};
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};
    use shared::types::page::PageRequest;

    #[tokio::test]
    async fn test_find_all_organizations_service_filtered_by_name() {
        let repo = InMemoryOrganizationRepository::new();
        for i in 1..=5 {
            let name = if i % 2 == 0 {
                format!("Acme Corp {}", i)
            } else {
                format!("Beta Ltd {}", i)
            };
            let org = Organization::new(
                OrganizationId::new(TSID::from(i as u64)),
                name,
                Document::new(format!("{:014}", i), DocumentType::Cnpj),
            )
            .with_audit(AuditFields::created("admin"));
            repo.save(&org).await.unwrap();
        }

        let service = FindAllOrganizationsService::new(repo);
        let request = FindAllOrganizationsRequest::with_name("Acme", PageRequest::new(1, 10));

        let result = service.execute(request).await;
        assert!(result.is_ok());
        let page = result.unwrap();
        assert_eq!(page.content.len(), 2);
        assert_eq!(page.total_elements, 2);
        assert_eq!(page.total_pages, 1);
        assert!(!page.has_next);
        assert!(!page.has_previous);
        for org in page.content {
            assert!(org.name().contains("Acme"));
        }
    }

    #[tokio::test]
    async fn test_find_all_organizations_service_pagination() {
        let repo = InMemoryOrganizationRepository::new();
        for i in 1..=10 {
            let org = Organization::new(
                OrganizationId::new(TSID::from(i as u64)),
                format!("Org {}", i),
                Document::new(format!("{:014}", i), DocumentType::Cnpj),
            )
            .with_audit(AuditFields::created("admin"));
            repo.save(&org).await.unwrap();
        }

        let service = FindAllOrganizationsService::new(repo);

        // Page 1, size 4 -> 4 items, total 10, total_pages 3, has_next true
        let req_p1 = FindAllOrganizationsRequest::without_filter(PageRequest::new(1, 4));
        let res_p1 = service.execute(req_p1).await.unwrap();
        assert_eq!(res_p1.content.len(), 4);
        assert_eq!(res_p1.total_elements, 10);
        assert_eq!(res_p1.total_pages, 3);
        assert!(res_p1.has_next);
        assert!(!res_p1.has_previous);

        // Page 2, size 4 -> 4 items, total 10, has_next true, has_previous true
        let req_p2 = FindAllOrganizationsRequest::without_filter(PageRequest::new(2, 4));
        let res_p2 = service.execute(req_p2).await.unwrap();
        assert_eq!(res_p2.content.len(), 4);
        assert_eq!(res_p2.total_elements, 10);
        assert!(res_p2.has_next);
        assert!(res_p2.has_previous);
        assert_ne!(res_p1.content[0].id(), res_p2.content[0].id());

        // Page 3, size 4 -> 2 items remaining, has_next false, has_previous true
        let req_p3 = FindAllOrganizationsRequest::without_filter(PageRequest::new(3, 4));
        let res_p3 = service.execute(req_p3).await.unwrap();
        assert_eq!(res_p3.content.len(), 2);
        assert_eq!(res_p3.total_elements, 10);
        assert!(!res_p3.has_next);
        assert!(res_p3.has_previous);
    }
}
