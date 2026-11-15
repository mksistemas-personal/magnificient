# Consumer conventions

This document describes how a message consumer is built inside Magnificient, based on the implementations in `organization-api` and `client-api`.

Use `organization-api`'s `ClientEventHandler` as the reference implementation. `client-api`'s `OrganizationEventHandler` is a simpler variant that handles a single event type. Both follow the same structural conventions.

## Reference implementation

When implementing a consumer, inspect these files first:

- Handler (organization-api): `apps/organization-api/src/adapters/consumer/client_event_handler.rs`
- Handler data: `apps/organization-api/src/adapters/consumer/client_event_handler/data.rs`
- Handler states: `apps/organization-api/src/adapters/consumer/client_event_handler/states.rs`
- Handler steps: `apps/organization-api/src/adapters/consumer/client_event_handler/steps.rs`
- Handler module: `apps/organization-api/src/adapters/consumer/mod.rs`
- Handler (client-api): `apps/client-api/src/adapters/consumer/organization_event_handler.rs`
- Handler data: `apps/client-api/src/adapters/consumer/organization_event_handler/data.rs`
- Handler states: `apps/client-api/src/adapters/consumer/organization_event_handler/states.rs`
- Handler steps: `apps/client-api/src/adapters/consumer/organization_event_handler/steps.rs`
- Consumer registry: `crates/messaging/src/consumer_registry.rs`
- Startup wiring: `apps/organization-api/src/main.rs` and `apps/client-api/src/main.rs`

## What is a consumer

A consumer is a component inside an API that listens to a RabbitMQ queue, deserializes incoming events, and processes them through a typed handler. It is the counterpart of a producer: one service publishes events via `RabbitMqPublisher`, and another service consumes them via `MessageConsumerRegistry`.

A consumer does not expose HTTP endpoints. It reacts to events from other services and updates its own domain state accordingly.

## Architecture overview

```
RabbitMQ queue ──► MessageConsumerRegistry ──► MessageHandler<Event> ──► railway steps ──► application use case
```

1. A message arrives on a RabbitMQ queue.
2. `MessageConsumerRegistry` reads the AMQP headers (`event_type`, `version`) and dispatches to the registered handler.
3. The handler implements `MessageHandler<Event>` and receives a deserialized event payload and metadata.
4. The handler runs a railway/typestate flow of steps: validate, resolve tenant, parse IDs, execute the business operation.
5. The final step delegates to an application use case, which persists the result.

## File structure

Place each event handler under `src/adapters/consumer/<event_name>_event_handler/`:

```
src/adapters/consumer/
├── mod.rs                                    # re-exports public handler types
└── <event_name>_event_handler/
    ├── <event_name>_event_handler.rs        # handler struct + MessageHandler impl
    ├── data.rs                               # context data structs per state
    ├── states.rs                             # state marker types
    └── steps.rs                              # validation and operation steps
```

The `mod.rs` declares the handler module and re-exports the public handler struct:

```rust
mod client_event_handler;

pub use client_event_handler::ClientEventHandler;
```

## Handler struct and MessageHandler trait

Create a struct that holds the API's `AppState` and implement `MessageHandler<Event>` from the `messaging` crate:

```rust
use async_trait::async_trait;
use messaging::{MessageHandler, MessageHandlerError, MessageMetadata};
use shared::entities::Context;

pub struct ClientEventHandler {
    app_state: AppState,
}

impl ClientEventHandler {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[async_trait]
impl MessageHandler<ClientEvent> for ClientEventHandler {
    async fn handle(
        &self,
        event: ClientEvent,
        metadata: MessageMetadata,
    ) -> Result<(), MessageHandlerError> {
        // dispatch by event_type, or run a single flow
    }
}
```

The `handle` method receives the deserialized event and the `MessageMetadata` extracted from AMQP headers. It should dispatch to the appropriate internal handler method based on `event.event_type`, or run a single railway flow if only one event type is handled.

For event types that the consumer does not handle, silently skip them (do not return an error):

```rust
match event.event_type {
    ClientEventType::AddClient => {
        self.handle_add_client(event, metadata).await?;
    }
    ClientEventType::DeleteClient => {
        self.handle_delete_client(event, metadata).await?;
    }
    _ => {}
}
Ok(())
```

## Railway/typestate flow

**Every consumer handler must use the railway/typestate flow**, following the existing implementations. This is the standard pattern, not an optional one for complex operations.

### `states.rs`

Define marker structs for each phase of the flow. Name them after the completed phase:

```rust
pub struct Initial;
pub struct EventValidated;
pub struct TenantResolved;
pub struct ClientIdParsed;
```

### `data.rs`

Define a data struct for each state. Each struct carries only the fields needed for the next step(s):

```rust
pub struct InitialContextData {
    pub event: ClientEvent,
    pub metadata: MessageMetadata,
}

pub struct EventValidatedContextData {
    pub client_id: String,
    pub organization_id: Option<String>,
    pub tenant: String,
}

pub struct TenantResolvedContextData {
    pub client_id: String,
    pub organization_id: Option<String>,
    pub tenant_state: AppState,
}

pub struct ClientIdParsedContextData {
    pub client_id: ClientId,
    pub organization_id: Option<OrganizationId>,
    pub tenant_state: AppState,
}
```

### `steps.rs`

Each step is a trait implemented for a specific `Context<State, Data>`. The trait method consumes the context and returns the next typed context (or the final result).

A typical step chain:

| Step | State transition | Purpose |
| --- | --- | --- |
| `validate_event` | `Initial` -> `EventValidated` | Check event type, extract required fields, validate tenant metadata |
| `resolve_tenant` | `EventValidated` -> `TenantResolved` | Look up the tenant and obtain a tenant-scoped `AppState` |
| `parse_*_id` | `TenantResolved` -> `*IdParsed` | Convert string IDs to typed domain IDs |
| execute operation | `*IdParsed` -> `Result<(), _>` | Call the application use case |

Example step:

```rust
pub trait ValidateEvent {
    fn validate_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError>;
}

impl ValidateEvent for Context<Initial, InitialContextData> {
    fn validate_event(
        self,
    ) -> Result<Context<EventValidated, EventValidatedContextData>, MessageHandlerError> {
        if self.data.event.event_type != ClientEventType::AddClient {
            return Err(MessageHandlerError::permanent(
                "Expected an AddClient event",
            ));
        }
        let tenant = self.data.metadata.require_tenant()?.to_owned();
        // extract and validate fields...
        Ok(Context::new(EventValidatedContextData {
            client_id: self.data.event.id,
            organization_id: Some(organization_id),
            tenant,
        }))
    }
}
```

Async steps (such as `resolve_tenant`) use `#[async_trait]`:

```rust
#[async_trait]
pub trait ResolveTenant {
    async fn resolve_tenant(
        self,
        state: &AppState,
    ) -> Result<Context<TenantResolved, TenantResolvedContextData>, MessageHandlerError>;
}
```

### Composing the flow

The handler's internal method composes the steps in order, using `?` for synchronous steps and `.await?` for asynchronous steps:

```rust
async fn handle_add_client(
    &self,
    event: ClientEvent,
    metadata: MessageMetadata,
) -> Result<(), MessageHandlerError> {
    use steps::{
        AddClientToOrganization, ParseClientAndOrganizationId,
        ResolveTenant, ValidateEvent,
    };

    Context::<Initial, _>::new(data::InitialContextData { event, metadata })
        .validate_event()?
        .resolve_tenant(&self.app_state)
        .await?
        .parse_client_and_organization_id()?
        .link_client_organization()
        .await
}
```

## Error handling

Return `MessageHandlerError` from every step:

- `MessageHandlerError::permanent(...)` — for malformed messages, unsupported event types, or invalid data. The message is discarded and not retried.
- `MessageHandlerError::retryable(...)` — for transient failures (e.g., database connection timeout). The message is requeued.

The `MessageHandlerError` type implements `From<BusinessError>` and `From<ApiError>`, so domain errors are automatically mapped: `BusinessError::Internal` becomes retryable; other variants become permanent.

## Tenant resolution

Multi-tenant APIs must resolve the tenant from the message metadata before accessing tenant-scoped resources. Use `metadata.require_tenant()` to extract the tenant string, then call `tenant_state(&app_state, &tenant)` to obtain the tenant-scoped `AppState`:

```rust
let tenant_state = tenant_state(&state, &self.data.tenant).await?;
```

The tenant value comes from the AMQP `tenant` header set by the producer. If the header is missing or empty, `require_tenant()` returns a permanent error.

## Registering the handler

In `main.rs`, create a `register_events` function that builds the `MessageConsumerRegistry` and registers each handler:

```rust
fn register_events(
    state: &AppState,
    consumer_client: RmqClient,
) -> Result<MessageConsumerRegistry, Box<dyn Error>> {
    let mut registry = MessageConsumerRegistry::new(consumer_client);
    registry.register::<ClientEvent, _>(
        EVENT_NAME,
        EVENT_VERSION,
        ClientEventHandler::new(state.clone()),
    )?;
    registry.start("organization-input.main");
    Ok(registry)
}
```

Key points:

- `EVENT_NAME` and `EVENT_VERSION` come from the producer's contracts crate (e.g., `client_contracts::v1::EVENT_NAME`).
- The queue name passed to `registry.start(...)` should follow the pattern `<service>-input.main`.
- Multiple event types can be registered on the same queue by calling `registry.register(...)` multiple times before `registry.start(...)`.
- The registry dispatches by matching the AMQP `event_type` and `version` headers against the registered keys.

## Startup wiring

The consumer registry is created during application startup alongside the HTTP server. The typical flow in `main.rs`:

```rust
let (state, consumer_registry) = setup_messaging(&config, db).await?;

let server_result = axum::serve(listener, create_router(state)).await;
consumer_registry.shutdown().await;
server_result?;
```

The consumer runs concurrently with the HTTP server. When the server stops, `consumer_registry.shutdown()` gracefully stops all consumers.

## Event contracts

- Depend on the producer's contracts crate (e.g., `client_contracts`, `organization_contracts`) for the event type, event enum, and constants.
- Do not define event types in the consuming API. The producer owns the contract.
- Import `EVENT_NAME` and `EVENT_VERSION` from the contracts crate to ensure the consumer registers with the same identifiers the producer uses.

## Tests expected for a new consumer

Add tests at the layer where behavior belongs:

- Step tests for validation logic (e.g., `validate_event` with correct/incorrect event types, missing tenant, missing IDs).
- Step tests for ID parsing with valid and invalid strings.
- Integration tests for the final operation step using an in-memory repository.
- Handler tests that verify the full flow from event to use case execution.

Prefer asserting the specific result or error rather than only checking `is_ok()` or `is_err()`.

## Checklist for adding a new consumer

1. Identify the event type from the producer's contracts crate and the event types to handle.
2. Create the handler directory under `src/adapters/consumer/<event_name>_event_handler/` with `data.rs`, `states.rs`, `steps.rs`, and the main handler file.
3. Define the state markers and context data structs for the railway flow.
4. Implement the validation, tenant resolution, ID parsing, and operation steps as traits on `Context<State, Data>`.
5. Implement `MessageHandler<Event>` and dispatch by event type.
6. Re-export the handler from `src/adapters/consumer/mod.rs`.
7. Register the handler in `main.rs` using `MessageConsumerRegistry::register(...)`.
8. Add the contracts crate dependency to `Cargo.toml` if not already present.
9. Add tests for each step and the full handler flow.
10. Format and check the narrow crate, then run relevant tests.

For lower-cost generation, pass this document plus the reference `organization-api` consumer files and the target API's current files.
