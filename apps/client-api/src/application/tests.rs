#[cfg(test)]
mod tests {
    use crate::application::{
        ClientPersistence, CreateClientRequest, CreateClientService, DeleteClientRequest,
        DeleteClientService, FindClientsRequest, FindClientsService, GetClientByIdRequest,
        GetClientByIdService, UpdateClientRequest, UpdateClientService,
    };
    use crate::domain::OrganizationId;
    use crate::infrastructure::InMemoryClientRepository;
    use shared::entities::BusinessError;
    use shared::entities::{Address, InMemoryAddressRepository};
    use shared::types::PageRequest;

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
        assert_eq!(created.nickname(), Some("Mo"));
        assert!(created.second_address().unwrap().id().is_some());
        assert!(created.audit().is_some());

        let updated = UpdateClientService::new(persistence.clone())
            .execute(UpdateClientRequest {
                id: created.id(),
                organization_id: created.organization_id(),
                name: "Morgan Lee Updated".to_owned(),
                nickname: None,
                second_address: None,
                updated_by: "editor".to_owned(),
            })
            .await
            .unwrap();
        assert_eq!(updated.name(), "Morgan Lee Updated");
        assert_eq!(updated.audit().unwrap().updated_by(), "editor");

        let page = FindClientsService::new(persistence.clone())
            .execute(FindClientsRequest {
                name: Some("Morgan".to_owned()),
                organization_id: Some(created.organization_id()),
                page_request: PageRequest::default(),
            })
            .await
            .unwrap();
        assert_eq!(page.total_elements, 1);

        DeleteClientService::new(persistence.clone())
            .execute(DeleteClientRequest {
                id: created.id(),
                updated_by: "deleter".to_owned(),
            })
            .await
            .unwrap();
        let missing = GetClientByIdService::new(persistence)
            .execute(GetClientByIdRequest { id: created.id() })
            .await;
        assert!(matches!(missing, Err(BusinessError::NotFound(_))));
    }
}
