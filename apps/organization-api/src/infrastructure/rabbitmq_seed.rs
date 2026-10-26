use lapin::types::{AMQPValue, FieldTable, LongString, ShortString};
use rabbit_warren::{ExchangeKind, RmqClient};
use std::error::Error;

pub struct RabbitmqSeed<'a> {
    pub client: &'a RmqClient,
}

impl<'a> RabbitmqSeed<'a> {
    pub fn new(client: &'a RmqClient) -> Self {
        Self { client }
    }

    pub async fn create(&self) -> Result<(), Box<dyn Error>> {
        self.client
            .declare_exchange("organization.input", ExchangeKind::Headers, true)
            .await?;
        self.client
            .declare_exchange("organization.output", ExchangeKind::Headers, true)
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
        Ok(())
    }
}
