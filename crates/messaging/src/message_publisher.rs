use async_trait::async_trait;
use rabbit_warren::RmqClient;
use serde::Serialize;
use shared::entities::BusinessError;

#[async_trait]
pub trait MessagePublisher: Send + Sync {
    async fn publish_json<T: Serialize + Send + Sync>(
        &self,
        exchange: &str,
        routing_key: &str,
        message: &T,
    ) -> Result<(), BusinessError>;
}

#[async_trait]
impl MessagePublisher for RabbitMqPublisher {
    async fn publish_json<T: Serialize + Send + Sync>(
        &self,
        exchange: &str,
        routing_key: &str,
        message: &T,
    ) -> Result<(), BusinessError> {
        self.client
            .publish_json(exchange, routing_key, message, None)
            .await
            .map_err(|e| BusinessError::Internal(format!("Erro ao publicar mensagem: {}", e)))
    }
}

#[derive(Clone)]
pub struct RabbitMqPublisher {
    client: RmqClient,
}

impl RabbitMqPublisher {
    pub fn new(client: RmqClient) -> Self {
        Self { client }
    }
}
