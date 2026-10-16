use crate::entities::BusinessError;
use async_trait::async_trait;

/// Trait para casos de uso síncronos de propósito geral.
pub trait UseCase<Request, Response> {
    fn execute(&self, request: Request) -> Response;
}

/// Trait para casos de uso síncronos de criação/adição de entidades que retornam `Result<Response, BusinessError>`.
pub trait AddEntityUseCase<Request, Response> {
    fn execute(&self, request: Request) -> Result<Response, BusinessError>;
}

/// Trait para casos de uso assíncronos de propósito geral.
#[async_trait]
pub trait AsyncUseCase<Request, Response>: Send + Sync {
    async fn execute(&self, request: Request) -> Response;
}

/// Trait para casos de uso assíncronos de criação/adição de entidades que retornam `Result<Response, BusinessError>`.
#[async_trait]
pub trait AsyncAddEntityUseCase<Request, Response>: Send + Sync {
    async fn execute(&self, request: Request) -> Result<Response, BusinessError>;
}

#[async_trait]
pub trait AsyncUpdateEntityUseCase<Request, Response>: Send + Sync {
    async fn execute(&self, request: Request) -> Result<Response, BusinessError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummySyncUseCase;
    impl UseCase<i32, i32> for DummySyncUseCase {
        fn execute(&self, request: i32) -> i32 {
            request * 2
        }
    }

    struct DummySyncAddEntityUseCase;
    impl AddEntityUseCase<String, String> for DummySyncAddEntityUseCase {
        fn execute(&self, request: String) -> Result<String, BusinessError> {
            if request.is_empty() {
                Err(BusinessError::validation("empty"))
            } else {
                Ok(format!("created:{request}"))
            }
        }
    }

    struct DummyAsyncUseCase;
    #[async_trait]
    impl AsyncUseCase<i32, i32> for DummyAsyncUseCase {
        async fn execute(&self, request: i32) -> i32 {
            request + 10
        }
    }

    struct DummyAsyncAddEntityUseCase;
    #[async_trait]
    impl AsyncAddEntityUseCase<String, String> for DummyAsyncAddEntityUseCase {
        async fn execute(&self, request: String) -> Result<String, BusinessError> {
            if request.is_empty() {
                Err(BusinessError::validation("empty async"))
            } else {
                Ok(format!("async_created:{request}"))
            }
        }
    }

    #[tokio::test]
    async fn test_use_case_traits() {
        let sync_uc = DummySyncUseCase;
        assert_eq!(sync_uc.execute(5), 10);

        let sync_add_uc = DummySyncAddEntityUseCase;
        assert_eq!(sync_add_uc.execute("test".to_string()).unwrap(), "created:test");
        assert!(sync_add_uc.execute("".to_string()).is_err());

        let async_uc = DummyAsyncUseCase;
        assert_eq!(async_uc.execute(5).await, 15);

        let async_add_uc = DummyAsyncAddEntityUseCase;
        assert_eq!(
            async_add_uc.execute("test".to_string()).await.unwrap(),
            "async_created:test"
        );
        assert!(async_add_uc.execute("".to_string()).await.is_err());

        // Test dynamic trait object
        let dyn_async_add_uc: std::sync::Arc<dyn AsyncAddEntityUseCase<String, String>> =
            std::sync::Arc::new(DummyAsyncAddEntityUseCase);
        assert_eq!(
            dyn_async_add_uc.execute("dyn".to_string()).await.unwrap(),
            "async_created:dyn"
        );
    }
}


