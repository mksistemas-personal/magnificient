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

pub struct MessageHeaders {
    event_type: String,
    source: String,
    message_id: String,
    version: i32,
    tenant: String,
}

impl MessageHeaders {
    pub fn new() -> Self {
        Self {
            event_type: String::new(),
            source: String::new(),
            message_id: Uuid::new_v4().to_string(),
            version: 0,
            tenant: String::new(),
        }
    }

    pub fn with_event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = event_type.into();
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    pub fn with_message_id(mut self, message_id: impl Into<String>) -> Self {
        self.message_id = message_id.into();
        self
    }

    pub fn with_version(mut self, version: i32) -> Self {
        self.version = version;
        self
    }

    pub fn with_tenant(mut self, tenant: String) -> Self {
        self.tenant = tenant;
        self
    }

    pub fn build(self) -> FieldTable {
        let mut headers = FieldTable::default();
        headers.insert(
            "message_id".into(),
            AMQPValue::LongString(self.message_id.into()),
        );
        headers.insert(
            "event_type".into(),
            AMQPValue::LongString(self.event_type.into()),
        );
        headers.insert("source".into(), AMQPValue::LongString(self.source.into()));
        headers.insert("version".into(), AMQPValue::LongInt(self.version));
        headers.insert("tenant".into(), AMQPValue::LongString(self.tenant.into()));
        headers
    }

    pub fn create_full(
        event_type: &str,
        source: &str,
        message_id: &str,
        version: i32,
    ) -> FieldTable {
        Self::new()
            .with_event_type(event_type)
            .with_source(source)
            .with_message_id(message_id)
            .with_version(version)
            .build()
    }

    pub fn create_default(
        event_type: &str,
        source: &str,
        version: i32,
        tenant: &str,
    ) -> FieldTable {
        Self::new()
            .with_event_type(event_type)
            .with_source(source)
            .with_version(version)
            .with_message_id(Uuid::new_v4().to_string())
            .with_tenant(tenant.to_string())
            .build()
    }
}
