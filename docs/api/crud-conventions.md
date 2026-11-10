# API CRUD conventions

This document describes the CRUD structure used by `organization-api` and the conventions to follow when adding or aligning CRUDs in `client-api` and `producer-api`.

Use `organization-api` as the implementation reference. The other APIs may have domain-specific fields and behavior, but should preserve the same separation of responsibilities and request flow. Do not copy organization-only behavior such as document rules, address persistence, tenant routing, or RabbitMQ events unless the target domain needs it.

## Reference implementation

When implementing a CRUD, inspect these files first:

- Domain: `apps/organization-api/src/domain/organization.rs`
- Repository contract: `apps/organization-api/src/domain/organization_repository.rs`
- Persistence and transaction policy: `apps/organization-api/src/application/persistence.rs`
- Use cases: `apps/organization-api/src/application/{add,update,delete,by_id,all}_usecase/`
- API DTOs and response mapping: `apps/organization-api/src/api/`
- Handlers and routes: `apps/organization-api/src/api/handlers/organizations_controller.rs` and `apps/organization-api/src/api/routes.rs`
- PostgreSQL implementation: `apps/organization-api/src/infrastructure/postgres_organization_repository.rs`
- In-memory implementation: `apps/organization-api/src/infrastructure/in_memory_organization_repository.rs`
- Application errors: `apps/organization-api/src/errors.rs`

## Layer responsibilities

Keep dependencies flowing inward: API code translates HTTP input and output, application code orchestrates use cases, domain code defines entities and repository contracts, and infrastructure implements persistence and integrations.

| Layer | Responsibility | Organization API example |
| --- | --- | --- |
| `api` | HTTP routes, request/query DTOs, response DTOs, handlers, HTTP error conversion | `add_organization_dto.rs`, `organizations_controller.rs` |
| `application` | Request models, validation and use-case flow, persistence coordination | `add_usecase/`, `OrganizationPersistence` |
| `domain` | Entity, typed ID, repository contract, domain events when needed | `organization.rs`, `organization_repository.rs` |
| `infrastructure` | PostgreSQL and in-memory repository implementations, external adapters | `postgres_organization_repository.rs` |

Do not put SQL in handlers or HTTP types in the domain. Keep domain behavior and validation independent of Axum and SQLx where practical.

## Domain entity and identifier

- Define an entity in `domain/<entity>.rs` and re-export it from `domain/mod.rs`.
- Use a marker type and `EntityId<Marker>` for a strongly typed identifier, following `OrganizationId`.
- Keep entity methods focused on domain state changes. Use builder-style methods such as `with_audit` only where they make construction clearer.
- Reuse shared types such as `AuditFields`, `Document`, `Address`, `TSID`, `Page`, and `BusinessError` when their semantics match.
- Include audit information when the entity is audited. Creation should set the creator; updates should preserve creation fields and update the updater and timestamp.
- Do not add fields, validation rules, or shared types just to make an entity look like `Organization`; model the actual domain.

## Repository contract and implementations

- Define one domain repository trait per aggregate in `domain/<entity>_repository.rs`.
- Repository methods receive `&mut PgConnection`. This lets the same contract be called with a pool-acquired connection or a transaction connection without duplicating transaction-specific methods.
- Keep methods focused on persistence operations: save, find by ID, delete, list/search, and count when pagination needs a total.
- Return domain entities and `BusinessError`, not SQLx rows or `ApiError`.
- Implement both PostgreSQL and in-memory adapters when the API uses the same repository abstraction in production and tests.
- Keep SQL and row-to-domain mapping inside the PostgreSQL repository. Prefer a small `FromRow` row type when the result has a joined or nontrivial mapping.
- Use bound query parameters. For dynamic filters, build the query with `QueryBuilder`; do not interpolate user input into SQL.
- Follow the existing soft-delete behavior where records must be retained: reads exclude deleted rows, delete marks the row deleted and updates audit fields, and a missing/already-deleted row returns the domain not-found error.
- Centralize domain error constants and constructors in an entity-specific error module, following `OrganizationBusinessErrors`. Do not duplicate error strings across handlers, services, and repositories.

The in-memory adapter should preserve the observable behavior of the PostgreSQL adapter for the operations it implements. It should not pretend to provide database rollback semantics.

## Application use cases

Create a request model and service for each operation that the API exposes. The organization API uses these operations:

- Create: validate input, create typed ID/domain values and audit fields, then persist.
- Update: validate, load the current entity, apply the allowed changes, update audit fields, then persist.
- Delete: validate the ID and actor, then perform a soft delete when required by the domain.
- Get by ID: validate the ID, load the entity, and propagate not-found errors.
- List: validate filters and pagination, retrieve the page content and total, then return `Page<Entity>`.

Use case services implement the shared async use-case traits and expose an `execute` method. **Every CRUD operation must use the railway/typestate flow**, following `organization-api`; this is the standard for new CRUDs and refactors, not an optional pattern for only complex operations. Do not put validation, entity changes, or persistence calls directly in the service's `execute` method.

### Required railway structure

Create one directory per use case under `application/`: `add_usecase/`, `update_usecase/`, `delete_usecase/`, `by_id_usecase/`, and `all_usecase/` (include only operations exposed by that API). Each directory must contain:

- `data.rs`: the operation request and the data carried between steps. Keep request data separate from intermediate results where the flow needs them.
- `states.rs`: marker types for the states of that operation. State transitions should be represented in the type of `Context<State, Data>`.
- `steps.rs`: validation and operation steps, each implemented for the context state it accepts and returning the next state/data (or the final result).
- A service file (`add_service.rs`, `update_service.rs`, `delete_service.rs`, `by_id_service.rs`, or `all_service.rs`): implement the shared use-case trait and compose the steps in order. The service owns/injects persistence and coordinates the railway; it does not reimplement step logic.
- `mod.rs`: keep implementation modules private and re-export the public request and service types needed by API wiring.

Use `Context::<Initial, _>::new(...)` to start a flow. Use `?` for synchronous steps and `.await?`/`.await` for asynchronous steps. A step must consume its input context and produce the next typed context/result; do not mutate a generic bag of optional fields or bypass the state transition. Name states after the completed phase (for example `Initial -> Validated -> AddressValidated -> Built`). Keep the states and data specific to each operation; share small domain helpers only when the behavior is truly common.

The minimum flow for each operation is:

| Operation | Typical railway |
| --- | --- |
| Create | `Initial -> Validated -> [RelatedDataValidated] -> Built -> save` |
| Update | `Initial -> Validated -> [RelatedDataValidated] -> Loaded -> Changed -> save` |
| Delete | `Initial -> Validated -> delete` |
| Get by ID | `Initial -> Validated -> find` |
| List/search | `Initial -> Validated -> find page` |

Bracketed phases apply only when that use case has related input to validate. Even a short operation gets its own `data.rs`, `states.rs`, and `steps.rs`, so every CRUD has the same entry point, separation of responsibilities, and discoverable flow. Do not add meaningless states: a state represents a completed validation or action and gates the next step.

Follow the concrete `organization-api` use-case modules as the reference for trait signatures, `Context` construction, async steps, and service composition. Keep business rules (such as client-specific optional address handling) inside the relevant step, not in a generic CRUD framework. API handlers should continue to call the service through its application request type.

Use `OrganizationPersistence` as the reference for sharing repository dependencies and transaction policy across services. Each API should have its own persistence coordinator typed to its own repository. Use `database::UnitOfWork` when an operation must be atomic across multiple repository calls. Repositories involved in that operation must use the connection passed by the coordinator so they participate in the same transaction.

Use the `FromPersistence` constructor pattern when several use cases share the same persistence dependencies. Wire production adapters in `api/state.rs`; inject in-memory adapters for unit/API tests.

## API DTOs, handlers, and routes

- Keep request and query DTOs in `api/`. Convert them into application request types at the API boundary.
- Keep response DTOs separate from domain entities. Map domain values to JSON explicitly, including typed IDs and audit timestamps.
- Apply defaults and aliases in DTO deserialization only when they are part of the API contract. Do not hide required-field validation behind a default value.
- Handlers should extract state/path/query/body, convert to an application request, call the use case, map the result to a response DTO, and choose the HTTP status.
- Convert `BusinessError` to `ApiError` at the API boundary. Do not return infrastructure errors directly to HTTP clients.
- Register endpoints in the API router and keep health routes consistent with the existing service.

The organization routes currently use a tenant path segment:

```text
POST   /api/v1/{tenant}/organizations
GET    /api/v1/{tenant}/organizations
GET    /api/v1/{tenant}/organizations/{id}
PUT    /api/v1/{tenant}/organizations/{id}
DELETE /api/v1/{tenant}/organizations/{id}
```

Preserve the target API's existing tenant model. If it is tenant-scoped, resolve the tenant through the trusted registry and use a tenant-scoped `UnitOfWork`; never treat an unchecked user-provided schema name as a trusted database identifier.

Follow the existing status convention: create returns `201 Created`, read/update return `200 OK`, and a successful delete returns `204 No Content`. Keep each API's established versioned prefix and resource naming consistent.

## Errors, events, and transactions

- Use entity-specific error constants/factories for expected business failures such as invalid data, duplicate values, and not found.
- Use `BusinessError::internal` for unexpected persistence failures; preserve the underlying error for logs/diagnostics, not as a misleading not-found response.
- Map domain errors to HTTP status codes through the shared API error conversion.
- Publish events only when the API/domain requires them. `organization-api` publishes create/update/delete events after its use case succeeds; this is not an automatic requirement for every CRUD.
- For an event-producing API, publish one event after each successful create, update, and delete use case. Keep the event enum/payload in `domain/message_events.rs`; use explicit payload DTOs, stable kebab-case event names, and include only fields intended for consumers. Delete events should at least carry the aggregate ID.
- Use the shared `messaging::RabbitMqConfig`, `create_rabbitmq_client`, `RabbitMqPublisher`, `MessagePublisher`, and `MessageHeaders` abstractions. Add RabbitMQ settings to the API config and its default TOML, declare the durable output exchange during startup, and inject the optional publisher through `AppState`. Tenant-scoped state must preserve the tenant value for message headers.
- Publish only after persistence succeeds and before returning the successful HTTP response, following `organization-api`. Map publish failures through the existing API error path. This preserves current behavior; it does not make publishing atomic with the database transaction.
- If an event and a database mutation must be atomic together, do not assume the current post-commit publish flow provides that guarantee. Use an explicitly designed outbox or equivalent mechanism if that guarantee is required.
- Keep transaction boundaries in the persistence/application coordination layer, not in HTTP handlers.

## Database migrations

- Put migrations for tables owned by reusable entities from `crates/shared` in `crates/shared/migrations/` and expose them through the shared crate's migrator.
- Keep service-specific migrations in that API's `migrations/` directory. When a service entity references a shared entity, put the foreign key, join column, and related index in the service migration; do not make the shared entity migration alter service-owned tables.
- Each API combines the shared migrator with its own migrator and runs the combined, version-ordered set using that API's migration history table (`organization_api_migrations` or `client_api_migrations`). Migration versions must be unique across the combined set.
- When the database is intentionally reset, keep migration setup simple: run the composed migrator from an empty schema and do not add legacy-history copying or reconciliation. Keep shared table creation idempotent with `CREATE TABLE IF NOT EXISTS`, since each API has a separate migration history and may execute the shared migration.

## Tests expected for a new CRUD

Add tests at the layer where behavior belongs:

- Domain tests for entity construction and state changes.
- Use-case tests for success, validation failures, not-found, and audit behavior, using in-memory repositories where possible.
- Repository tests for SQL mapping and persistence behavior when the project has a database-backed test setup.
- DTO tests for conversion/defaults and serialization where the API contract needs them.
- Handler/router tests for HTTP method, path, status, response shape, and error mapping.
- Tests for transaction behavior when an operation coordinates multiple repositories.

Prefer asserting the specific result or error rather than only checking `is_ok()` or `is_err()`.

## Checklist for AI-assisted CRUD generation

Before changing code, inspect the reference files listed above and the target API's existing modules. Then:

1. Identify the entity fields, invariants, audit requirements, soft-delete policy, tenant scope, and any domain events from the task or existing code. Do not invent missing business rules.
2. Add or update the domain entity, typed ID, repository contract, and entity error constants.
3. Add a railway module per required CRUD operation, each with `data.rs`, `states.rs`, `steps.rs`, a service file, and `mod.rs`; compose the steps in the service and reuse shared pagination, error, audit, and persistence types where suitable.
4. Add PostgreSQL and in-memory adapters that implement the same contract.
5. Add input/query/response DTOs, conversions, handlers, routes, and `AppState` wiring.
6. Add migrations only when the schema needs a change; follow the project's existing migration naming and conventions.
7. Add required events only when specified or already established for that domain.
8. Add the layer-appropriate tests from the checklist above.
9. Format and check the narrow crate, then run relevant tests when the task's verification scope calls for them; report any unavailable integration dependencies instead of claiming they passed.
10. Summarize created/changed files, API routes, assumptions, and verification results.

For lower-cost generation, pass this document plus only the relevant `organization-api` reference files and the target API's current files. Avoid including unrelated APIs, deployment manifests, or the entire repository unless the task requires them.
