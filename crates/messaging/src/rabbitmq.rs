use rabbit_warren::{Backoff, ConnectOptions, ReconnectPolicy, RetryConfig, RmqClient};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RabbitMqConfig {
    pub rabbitmq_url: String,
    pub rabbitmq_prefetch_count: u16,
    pub rabbitmq_reconnect_max_attempts: u32,
    pub rabbitmq_reconnect_initial_ms: u64,
    pub rabbitmq_reconnect_max_ms: u64,
    pub rabbitmq_retry_max_attempts: u32,
    pub rabbitmq_retry_initial_ms: u64,
    pub rabbitmq_retry_max_ms: u64,
}

pub struct RabbitMq {
    client: RmqClient,
}

pub async fn create_rabbitmq_client(
    config: &RabbitMqConfig,
) -> Result<RmqClient, rabbit_warren::Error> {
    let options = ConnectOptions {
        prefetch_count: config.rabbitmq_prefetch_count,

        reconnect: ReconnectPolicy {
            enabled: true,

            max_attempts: config.rabbitmq_reconnect_max_attempts,

            backoff: Backoff {
                initial_ms: config.rabbitmq_reconnect_initial_ms,

                max_ms: config.rabbitmq_reconnect_max_ms,

                multiplier: 2.0,

                jitter: true,
            },
        },

        retry: RetryConfig {
            // configurar conforme a API da versão utilizada
            ..Default::default()
        },
    };

    RmqClient::connect_with_options(&config.rabbitmq_url, options).await
}

impl RabbitMq {
    pub async fn new(config: &RabbitMqConfig) -> Result<Self, rabbit_warren::Error> {
        let client = create_rabbitmq_client(config).await?;

        Ok(Self { client })
    }

    pub fn client(&self) -> &RmqClient {
        &self.client
    }
}
