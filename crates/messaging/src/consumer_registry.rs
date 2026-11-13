use async_trait::async_trait;
use lapin::{message::Delivery, types::AMQPValue};
use rabbit_warren::{Consumer, ProcessingError, RmqClient};
use serde::de::DeserializeOwned;
use shared::entities::BusinessError;
use shared::errors::api_error::ApiError;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageMetadata {
    pub event_type: String,
    pub version: i32,
    pub message_id: Option<String>,
    pub source: Option<String>,
    pub tenant: Option<String>,
}

impl MessageMetadata {
    pub fn require_tenant(&self) -> Result<&str, MessageHandlerError> {
        self.tenant
            .as_deref()
            .map(str::trim)
            .filter(|tenant| !tenant.is_empty())
            .ok_or_else(|| {
                MessageHandlerError::permanent("Message is missing valid tenant metadata")
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureStrategy {
    Retry,
    Discard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageHandlerError {
    pub message: String,
    pub strategy: FailureStrategy,
}

impl MessageHandlerError {
    pub fn retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            strategy: FailureStrategy::Retry,
        }
    }

    pub fn permanent(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            strategy: FailureStrategy::Discard,
        }
    }
}

impl From<ApiError> for MessageHandlerError {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::Internal(_) => Self::retryable(error.to_string()),
            _ => Self::permanent(error.to_string()),
        }
    }
}

impl From<BusinessError> for MessageHandlerError {
    fn from(error: BusinessError) -> Self {
        match error {
            BusinessError::Internal(_) => Self::retryable(error.to_string()),
            _ => Self::permanent(error.to_string()),
        }
    }
}

#[async_trait]
pub trait MessageHandler<E>: Send + Sync + 'static
where
    E: Send + 'static,
{
    async fn handle(&self, event: E, metadata: MessageMetadata) -> Result<(), MessageHandlerError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct HandlerKey {
    event_type: String,
    version: i32,
}

#[async_trait]
trait ErasedMessageHandler: Send + Sync {
    async fn dispatch(
        &self,
        payload: Vec<u8>,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError>;
}

struct TypedMessageHandler<E, H> {
    handler: H,
    marker: std::marker::PhantomData<fn() -> E>,
}

#[async_trait]
impl<E, H> ErasedMessageHandler for TypedMessageHandler<E, H>
where
    E: DeserializeOwned + Send + Sync + 'static,
    H: MessageHandler<E>,
{
    async fn dispatch(
        &self,
        payload: Vec<u8>,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        let event = serde_json::from_slice::<E>(&payload)
            .map_err(|error| MessageHandlerError::permanent(error.to_string()))?;
        self.handler.handle(event, metadata).await
    }
}

/// Explicit event-to-handler registry for a queue. Event type and schema
/// version come from AMQP headers; each registered handler receives a typed
/// deserialized payload and the message metadata.
pub struct MessageConsumerRegistry {
    client: RmqClient,
    handlers: HashMap<HandlerKey, Arc<dyn ErasedMessageHandler>>,
    consumers: Vec<Consumer>,
}

impl MessageConsumerRegistry {
    pub fn new(client: RmqClient) -> Self {
        Self {
            client,
            handlers: HashMap::new(),
            consumers: Vec::new(),
        }
    }

    pub fn register<E, H>(
        &mut self,
        event_type: impl Into<String>,
        version: i32,
        handler: H,
    ) -> Result<(), MessageConsumerRegistryError>
    where
        E: DeserializeOwned + Send + Sync + 'static,
        H: MessageHandler<E>,
    {
        let key = HandlerKey {
            event_type: event_type.into(),
            version,
        };
        if self.handlers.contains_key(&key) {
            return Err(MessageConsumerRegistryError {
                event_type: key.event_type,
                version: key.version,
            });
        }
        self.handlers.insert(
            key,
            Arc::new(TypedMessageHandler::<E, H> {
                handler,
                marker: std::marker::PhantomData,
            }),
        );
        Ok(())
    }

    /// Starts dispatching messages from `queue`. Successful handlers are acked;
    /// transient errors are retried and permanent errors are discarded.
    pub fn start(&mut self, queue: &str) {
        let handlers = self.handlers.clone();
        let consumer = self.client.consume(queue, move |delivery| {
            let handlers = handlers.clone();
            async move {
                let metadata = match MessageMetadata::from_delivery(&delivery) {
                    Ok(metadata) => metadata,
                    Err(error) => return Err(ProcessingError::permanent(error)),
                };
                let key = HandlerKey {
                    event_type: metadata.event_type.clone(),
                    version: metadata.version,
                };
                let Some(handler) = handlers.get(&key) else {
                    return Err(ProcessingError::permanent(format!(
                        "No message handler registered for event '{}' version {}",
                        key.event_type, key.version
                    )));
                };
                match handler.dispatch(delivery.data, metadata).await {
                    Ok(()) => Ok(()),
                    Err(error) => match error.strategy {
                        FailureStrategy::Retry => Err(ProcessingError::retryable(error.message)),
                        FailureStrategy::Discard => Err(ProcessingError::permanent(error.message)),
                    },
                }
            }
        });
        self.consumers.push(consumer);
    }

    /// Shuts down every consumer started by this registry.
    pub async fn shutdown(self) {
        for consumer in self.consumers {
            consumer.shutdown().await;
        }
    }
}

impl MessageMetadata {
    fn from_delivery(delivery: &Delivery) -> Result<Self, String> {
        let headers = delivery
            .properties
            .headers()
            .as_ref()
            .ok_or_else(|| "Message is missing AMQP headers".to_owned())?;
        let values = headers.inner();
        let event_type = long_string_header(values, "event_type")?
            .ok_or_else(|| "Message is missing a valid event_type header".to_owned())?;
        let version = values
            .get("version")
            .and_then(AMQPValue::as_long_int)
            .ok_or_else(|| "Message is missing a valid version header".to_owned())?;

        Ok(Self {
            event_type,
            version,
            message_id: long_string_header(values, "message_id")?,
            source: long_string_header(values, "source")?,
            tenant: long_string_header(values, "tenant")?,
        })
    }
}

fn long_string_header(
    headers: &std::collections::BTreeMap<lapin::types::ShortString, AMQPValue>,
    name: &str,
) -> Result<Option<String>, String> {
    match headers.get(name) {
        Some(AMQPValue::LongString(value)) => Ok(Some(
            String::from_utf8(value.as_bytes().to_vec())
                .map_err(|error| format!("Header '{name}' is not valid UTF-8: {error}"))?,
        )),
        Some(_) => Err(format!("Header '{name}' has an invalid value type")),
        None => Ok(None),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageConsumerRegistryError {
    pub event_type: String,
    pub version: i32,
}

impl fmt::Display for MessageConsumerRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "A handler for event '{}' version {} is already registered",
            self.event_type, self.version
        )
    }
}

impl std::error::Error for MessageConsumerRegistryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::entities::BusinessError;
    use shared::errors::api_error::ApiError;

    #[test]
    fn test_message_metadata_require_tenant() {
        let metadata = MessageMetadata {
            event_type: "client.created".to_string(),
            version: 1,
            message_id: None,
            source: None,
            tenant: Some("  tenant-a  ".to_string()),
        };
        assert_eq!(metadata.require_tenant().unwrap(), "tenant-a");

        let empty_metadata = MessageMetadata {
            event_type: "client.created".to_string(),
            version: 1,
            message_id: None,
            source: None,
            tenant: Some("   ".to_string()),
        };
        assert!(empty_metadata.require_tenant().is_err());

        let none_metadata = MessageMetadata {
            event_type: "client.created".to_string(),
            version: 1,
            message_id: None,
            source: None,
            tenant: None,
        };
        assert!(none_metadata.require_tenant().is_err());
    }

    #[test]
    fn test_from_api_error() {
        let internal = ApiError::Internal("db failed".to_string());
        let handler_error: MessageHandlerError = internal.into();
        assert_eq!(handler_error.strategy, FailureStrategy::Retry);

        let not_found = ApiError::NotFound("item".to_string());
        let handler_error: MessageHandlerError = not_found.into();
        assert_eq!(handler_error.strategy, FailureStrategy::Discard);
    }

    #[test]
    fn test_from_business_error() {
        let internal = BusinessError::Internal("timeout".to_string());
        let handler_error: MessageHandlerError = internal.into();
        assert_eq!(handler_error.strategy, FailureStrategy::Retry);

        let not_found = BusinessError::NotFound("entity".to_string());
        let handler_error: MessageHandlerError = not_found.into();
        assert_eq!(handler_error.strategy, FailureStrategy::Discard);
    }
}
