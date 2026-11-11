// Módulo de mensageria

pub mod consumer_registry;
pub mod message_publisher;
pub mod rabbitmq;

pub use consumer_registry::{
    FailureStrategy, MessageConsumerRegistry, MessageConsumerRegistryError, MessageHandler,
    MessageHandlerError, MessageMetadata,
};
pub use message_publisher::{MessageHeaders, MessagePublisher, RabbitMqPublisher};
pub use rabbitmq::{RabbitMq, RabbitMqConfig, create_rabbitmq_client};
