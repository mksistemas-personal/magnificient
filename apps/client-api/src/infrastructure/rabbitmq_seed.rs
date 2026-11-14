use crate::constants::{
    CLIENT_INPUT_EXCHANGE, CLIENT_INPUT_QUEUE, CLIENT_OUTPUT_EXCHANGE, ORGANIZATION_EVENT,
    ORGANIZATION_OUTPUT_EXCHANGE, ROUTING_KEY,
};
use lapin::options::{ExchangeBindOptions, QueueBindOptions};
use lapin::types::{AMQPValue, FieldTable, LongString, ShortString};
use rabbit_warren::{ExchangeKind, RmqClient};
use std::error::Error;

pub struct RabbitmqSeed<'a> {
    client: &'a RmqClient,
}

impl<'a> RabbitmqSeed<'a> {
    pub fn new(client: &'a RmqClient) -> Self {
        Self { client }
    }

    pub async fn create(&self) -> Result<(), Box<dyn Error>> {
        self.client
            .declare_exchange(CLIENT_INPUT_EXCHANGE, ExchangeKind::Headers, true)
            .await?;
        self.client
            .declare_exchange(CLIENT_OUTPUT_EXCHANGE, ExchangeKind::Headers, true)
            .await?;

        let mut arguments = FieldTable::default();

        arguments.insert(
            ShortString::from("x-queue-type"),
            AMQPValue::LongString(LongString::from("quorum")),
        );

        self.client
            .declare_queue_with_arguments(
                CLIENT_INPUT_QUEUE,
                true, // durable
                arguments,
            )
            .await?;

        let channel = self.client.get_channel().await?;
        let binding_arguments = organization_event_binding_arguments();
        channel
            .exchange_bind(
                CLIENT_INPUT_EXCHANGE.into(),
                ORGANIZATION_OUTPUT_EXCHANGE.into(),
                ROUTING_KEY.into(),
                ExchangeBindOptions::default(),
                binding_arguments.clone(),
            )
            .await?;
        channel
            .queue_bind(
                CLIENT_INPUT_QUEUE.into(),
                CLIENT_INPUT_EXCHANGE.into(),
                ROUTING_KEY.into(),
                QueueBindOptions::default(),
                binding_arguments,
            )
            .await?;
        Ok(())
    }
}

fn organization_event_binding_arguments() -> FieldTable {
    let mut arguments = FieldTable::default();
    arguments.insert(
        ShortString::from("x-match"),
        AMQPValue::LongString(LongString::from("all")),
    );
    arguments.insert(
        ShortString::from("event_type"),
        AMQPValue::LongString(LongString::from(ORGANIZATION_EVENT)),
    );
    arguments
}
