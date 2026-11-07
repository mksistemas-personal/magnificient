use crate::application::OrganizationPersistence;
use crate::application::all_usecase::data::*;
use crate::application::all_usecase::states::*;
use crate::application::all_usecase::steps::{FindOrganizationsContext, ValidateInitialContext};
use crate::application::service_constructors::FromPersistence;
use crate::domain::Organization;
use async_trait::async_trait;
use shared::entities::BusinessError;
use shared::entities::use_case::context::Context;
use shared::entities::use_case::use_case_traits::AsyncGetAllUseCase;
use shared::types::page::Page;

#[derive(Clone)]
pub struct FindAllOrganizationsService {
    persistence: OrganizationPersistence,
}

impl FromPersistence for FindAllOrganizationsService {
    fn with_persistence(persistence: OrganizationPersistence) -> Self {
        Self { persistence }
    }
}

impl FindAllOrganizationsService {
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
            .find(&self.persistence)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service_constructors::FromPersistence;
    use crate::domain::{Organization, OrganizationId, OrganizationRepository};
    use crate::infrastructure::InMemoryOrganizationRepository;
    use shared::entities::{AuditFields, Document, DocumentType, TSID};
    use shared::types::page::PageRequest;

    async fn seed_repos(repo: &InMemoryOrganizationRepository, count: usize) {
        let conn: &mut sqlx::PgConnection =
            unsafe { &mut *std::ptr::NonNull::dangling().as_ptr() };
        for i in 1..=count {
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
            OrganizationRepository::save(repo, conn, &org)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_find_all_organizations_service_filtered_by_name() {
        let repo = InMemoryOrganizationRepository::new();
        seed_repos(&repo, 5).await;

        let service = FindAllOrganizationsService::new_for_tests(repo);
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
        seed_repos(&repo, 10).await;

        let service = FindAllOrganizationsService::new_for_tests(repo);

        let req_p1 = FindAllOrganizationsRequest::without_filter(PageRequest::new(1, 4));
        let res_p1 = service.execute(req_p1).await.unwrap();
        assert_eq!(res_p1.content.len(), 4);
        assert_eq!(res_p1.total_elements, 10);
        assert_eq!(res_p1.total_pages, 3);
        assert!(res_p1.has_next);
        assert!(!res_p1.has_previous);

        let req_p2 = FindAllOrganizationsRequest::without_filter(PageRequest::new(2, 4));
        let res_p2 = service.execute(req_p2).await.unwrap();
        assert_eq!(res_p2.content.len(), 4);
        assert!(res_p2.has_next);
        assert!(res_p2.has_previous);
        assert_ne!(res_p1.content[0].id(), res_p2.content[0].id());

        let req_p3 = FindAllOrganizationsRequest::without_filter(PageRequest::new(3, 4));
        let res_p3 = service.execute(req_p3).await.unwrap();
        assert_eq!(res_p3.content.len(), 2);
        assert!(!res_p3.has_next);
        assert!(res_p3.has_previous);
    }
}
