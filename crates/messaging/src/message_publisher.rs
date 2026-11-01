use async_trait::async_trait;
use lapin::types::AMQPValue;
use rabbit_warren::{FieldTable, RmqClient};
use serde::Serialize;
use shared::entities::BusinessError;
use uuid::Uuid;

#[async_trait]
pub trait MessagePublisher: Send + Sync {
    async fn publish_json<T: Serialize + Send + Sync>(
        &self,
        exchange: &str,
        routing_key: &str,
        message: &T,
        headers: Option<FieldTable>,
    ) -> Result<(), BusinessError>;
}

#[async_trait]
impl MessagePublisher for RabbitMqPublisher {
    async fn publish_json<T: Serialize + Send + Sync>(
        &self,
        exchange: &str,
        routing_key: &str,
        message: &T,
        headers: Option<FieldTable>,
    ) -> Result<(), BusinessError> {
        self.client
            .publish_json(exchange, routing_key, message, headers)
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

pub struct MessageHeaders;

impl MessageHeaders {
    pub fn create_full(
        event_type: &str,
        source: &str,
        message_id: &str,
        version: i32,
    ) -> FieldTable {
        let mut headers = FieldTable::default();
        headers.insert(
            "message_id".into(),
            AMQPValue::LongString(message_id.into()),
        );
        headers.insert(
            "event_type".into(),
            AMQPValue::LongString(event_type.into()),
        );
        headers.insert("source".into(), AMQPValue::LongString(source.into()));
        headers.insert("version".into(), AMQPValue::LongInt(version));
        headers
    }

    pub fn create_default(event_type: &str, source: &str, version: i32) -> FieldTable {
        let mut headers = FieldTable::default();
        headers.insert(
            "message_id".into(),
            AMQPValue::LongString(Uuid::new_v4().to_string().into()),
        );
        headers.insert(
            "event_type".into(),
            AMQPValue::LongString(event_type.into()),
        );
        headers.insert("source".into(), AMQPValue::LongString(source.into()));
        headers.insert("version".into(), AMQPValue::LongInt(version));
        headers
    }
}
