use lapin::Channel;
use lapin::options::{ExchangeBindOptions, QueueBindOptions};
use lapin::types::{AMQPValue, FieldTable, LongString, ShortString};
use rabbit_warren::{ExchangeKind, RmqClient};
use std::error::Error;
use std::sync::Arc;

const CLIENT_EVENT: &str = "ClientEvent";
const ORGANIZATION_INPUT_EXCHANGE: &str = "organization.input";
const ORGANIZATION_OUTPUT_EXCHANGE: &str = "organization.output";
const CLIENT_OUTPUT_EXCHANGE: &str = "client.output";
const ORGANIZATION_INPUT_QUEUE: &str = "organization-input.main";
const ROUTING_KEY: &str = "*";

pub struct RabbitmqSeed<'a> {
    pub client: &'a RmqClient,
}

impl<'a> RabbitmqSeed<'a> {
    pub fn new(client: &'a RmqClient) -> Self {
        Self { client }
    }

    pub async fn create(&self) -> Result<(), Box<dyn Error>> {
        self.client
            .declare_exchange(ORGANIZATION_INPUT_EXCHANGE, ExchangeKind::Headers, true)
            .await?;
        self.client
            .declare_exchange(ORGANIZATION_OUTPUT_EXCHANGE, ExchangeKind::Headers, true)
            .await?;

        let mut arguments = FieldTable::default();

        arguments.insert(
            ShortString::from("x-queue-type"),
            AMQPValue::LongString(LongString::from("quorum")),
        );

        self.client
            .declare_queue_with_arguments(
                "organization-input.main",
                true, // durable
                arguments,
            )
            .await?;

        let channel = self.client.get_channel().await?;
        Self::bind_client_with_organization(channel).await?;

        Ok(())
    }

    fn client_event_binding_arguments() -> FieldTable {
        let mut arguments = FieldTable::default();
        arguments.insert(
            ShortString::from("x-match"),
            AMQPValue::LongString(LongString::from("all")),
        );
        arguments.insert(
            ShortString::from("event_type"),
            AMQPValue::LongString(LongString::from(CLIENT_EVENT)),
        );
        arguments
    }
    async fn bind_client_with_organization(channel: Arc<Channel>) -> Result<(), Box<dyn Error>> {
        let binding_arguments = Self::client_event_binding_arguments();
        channel
            .exchange_bind(
                ORGANIZATION_INPUT_EXCHANGE.into(),
                CLIENT_OUTPUT_EXCHANGE.into(),
                ROUTING_KEY.into(),
                ExchangeBindOptions::default(),
                binding_arguments.clone(),
            )
            .await?;
        channel
            .queue_bind(
                ORGANIZATION_INPUT_QUEUE.into(),
                ORGANIZATION_INPUT_EXCHANGE.into(),
                ROUTING_KEY.into(),
                QueueBindOptions::default(),
                binding_arguments,
            )
            .await?;
        Ok(())
    }
}
