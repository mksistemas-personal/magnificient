use lapin::message::Delivery;
use organization_contracts::v1::{
    EVENT_NAME, EVENT_VERSION, OrganizationEvent, OrganizationEventType,
};
use rabbit_warren::{Consumer, ProcessingError, RmqClient, deserialize_delivery};
use tracing::{error, info};

const ORGANIZATION_EVENT_QUEUE: &str = "client-input.main";

/// Consumes Organization events routed to the Client API input queue.
pub struct OrganizationEventConsumer {
    consumer: Consumer,
}

impl OrganizationEventConsumer {
    pub fn start(client: &RmqClient) -> Self {
        let consumer = client.consume(ORGANIZATION_EVENT_QUEUE, |delivery| async move {
            if let Err(error) = validate_event_headers(&delivery) {
                error!(%error, "Discarding OrganizationEvent with unsupported headers");
                return Err(ProcessingError::permanent(error));
            }

            let event = match deserialize_delivery::<OrganizationEvent>(&delivery) {
                Ok(event) => event,
                Err(error) => {
                    error!(%error, "Discarding invalid OrganizationEvent payload");
                    return Err(ProcessingError::permanent(error.to_string()));
                }
            };

            process_organization_event(event);
            Ok::<(), ProcessingError<String>>(())
        });

        Self { consumer }
    }

    pub async fn shutdown(self) {
        self.consumer.shutdown().await;
    }
}

fn validate_event_headers(delivery: &Delivery) -> Result<(), String> {
    let headers = delivery
        .properties
        .headers()
        .as_ref()
        .ok_or_else(|| "Message is missing AMQP headers".to_owned())?;
    let event_name = headers
        .inner()
        .get("event_type")
        .and_then(|value| value.as_long_string())
        .ok_or_else(|| "Message is missing a valid event_type header".to_owned())?;
    if event_name.as_bytes() != EVENT_NAME.as_bytes() {
        return Err(format!(
            "Unsupported event type '{}'",
            String::from_utf8_lossy(event_name.as_bytes())
        ));
    }

    let version = headers
        .inner()
        .get("version")
        .and_then(|value| value.as_long_int())
        .ok_or_else(|| "Message is missing a valid version header".to_owned())?;
    if version != EVENT_VERSION {
        return Err(format!("Unsupported OrganizationEvent version '{version}'"));
    }
    Ok(())
}

/// The Client API currently has no Organization read model to update; successful
/// deserialization and dispatch are acknowledged after this event is observed.
fn process_organization_event(event: OrganizationEvent) {
    match event.event_type {
        OrganizationEventType::AddOrganization => {
            info!(organization_id = %event.id, "Received OrganizationEvent add");
        }
        OrganizationEventType::UpdateOrganization => {
            info!(organization_id = %event.id, "Received OrganizationEvent update");
        }
        OrganizationEventType::DeleteOrganization => {
            info!(organization_id = %event.id, "Received OrganizationEvent delete");
        }
    }
}
