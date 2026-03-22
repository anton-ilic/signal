# SIGNAL Rust Backend

This directory contains the Rust backend for the SIGNAL wireless call-button system.

The backend is the cloud-connected part of the system. It accepts button press events from authenticated receivers, stores them in PostgreSQL, deduplicates replayed events, and exposes APIs for the app to manage devices and read event history.

The higher-level product architecture is documented in [docs/architecture/ARCHITECTURE.md](/Users/anton/Dev/signal/docs/architecture/ARCHITECTURE.md).

## What The Backend Does

The backend sits between the receiver hardware and the mobile app:

`button -> receiver -> backend -> app`

Its responsibilities are:

- authenticate receivers
- ingest button press events
- store event history
- prevent replay attacks using `event_counter`
- manage buttons and receivers
- store push tokens for app devices
- prepare the path for push notification delivery

In the current implementation, push delivery is a logging stub. The backend records everything needed for later APNs/FCM integration, but it does not yet call those services directly.

## Tech Stack

- Rust
- Axum
- Tokio
- PostgreSQL
- SQLx
- Serde
- Tracing

## Directory Layout

```text
backend/
  Cargo.toml
  Dockerfile
  docker-compose.yml
  .env.example
  migrations/
    0001_initial.sql
  src/
    main.rs
    lib.rs
    config.rs
    error.rs
    domain/
    db/
    services/
    middleware/
    routes/
  tests/
```

## How The Backend Is Structured

### `src/main.rs`

Application entrypoint.

- reads environment-based configuration
- initializes tracing
- creates the PostgreSQL pool
- runs SQLx migrations on startup
- builds shared application state
- starts the Axum HTTP server

### `src/lib.rs`

Library entrypoint used by the binary and tests.

- defines `AppState`
- wires the shared store and notification service into the router

### `src/config.rs`

Configuration parsing.

It reads:

- `DATABASE_URL`
- `SIGNAL_BACKEND_HOST`
- `SIGNAL_BACKEND_PORT`
- `DATABASE_MAX_CONNECTIONS`
- `CORS_ALLOW_ORIGIN`
- `RUST_LOG`

### `src/error.rs`

Central application error type.

- converts domain and database errors into HTTP responses
- avoids leaking raw SQL errors to clients
- logs server-side failures through `tracing`

### `src/domain/`

Domain and API types.

This module defines:

- persisted models such as `User`, `Receiver`, `Button`, `ButtonEvent`, and `PushToken`
- request payloads
- response payloads
- helper structs used when creating new records

### `src/db/`

Persistence boundary.

`db/mod.rs` defines the `Store` trait, which is the backend's database abstraction.  
`db/postgres.rs` implements that trait with SQLx and PostgreSQL.

This separation keeps the business logic from depending directly on raw SQL and makes the event service testable with in-memory fakes.

### `src/services/`

Business logic layer.

`services/events.rs` contains the core button press ingestion workflow:

1. validate the request
2. load the button
3. verify that the button belongs to the authenticated receiver
4. check whether the `(button_id, event_counter)` pair already exists
5. insert a new event if it does not
6. load the user's push tokens
7. invoke the notification sender

`services/notifications.rs` defines the notification interface and a logging implementation.

### `src/middleware/`

Authentication helpers.

Current auth model:

- receivers authenticate with `Authorization: Bearer <token>`
- app-facing routes currently use `x-user-id: <uuid>` as a development placeholder

This module validates headers and resolves the associated user or receiver before route handlers continue.

### `src/routes/`

HTTP transport layer.

The routes map incoming HTTP requests into service or store calls:

- `health.rs`: health and root status endpoints
- `users.rs`: create or fetch the development user identity by email
- `devices.rs`: create/list receivers and buttons, plus receiver heartbeat
- `events.rs`: ingest receiver button presses and list app-visible event history
- `push_tokens.rs`: register app push tokens

## Startup Flow

When the backend starts:

1. configuration is loaded from environment variables
2. tracing is configured
3. a PostgreSQL connection pool is created
4. the migration in `migrations/0001_initial.sql` is applied
5. the router is built with shared state
6. the Axum server begins listening for HTTP traffic

This means the service is self-bootstrapping with respect to schema changes, as long as the database already exists and is reachable.

## Database Schema

The initial migration creates these tables:

### `users`

- user identity
- email
- creation timestamp

### `receivers`

- receiver metadata
- owning user
- bearer auth token
- last heartbeat timestamp

### `buttons`

- button id
- owning user
- paired receiver
- human-readable label

### `button_events`

- immutable button press records
- button id
- receiver id
- event counter
- pressed timestamp
- received timestamp

The key replay-protection rule is the unique constraint on:

- `(button_id, event_counter)`

That guarantees the backend will not store the same logical button press twice.

### `push_tokens`

- app device push tokens
- owning user
- platform (`ios` or `android`)

## Event Ingestion Flow

The most important backend flow is `POST /v1/events/button-press`.

### Step 1: Receiver authenticates

The receiver sends:

```http
Authorization: Bearer <receiver-auth-token>
```

The backend loads the receiver from the database. If the token is invalid, the request is rejected with `401 Unauthorized`.

### Step 2: Payload is validated

The payload looks like:

```json
{
  "button_id": "button-123",
  "event_counter": 42,
  "pressed_at": "2026-03-22T17:00:00Z",
  "received_at": "2026-03-22T17:00:01Z"
}
```

Validation rules:

- `button_id` must be present
- `event_counter` must be non-negative
- the button must exist
- the button must be paired with the authenticated receiver

### Step 3: Replay protection runs

Before inserting a new record, the backend checks whether a row already exists for the same `button_id` and `event_counter`.

If it finds one, it returns the existing event instead of creating a new one. This makes the endpoint idempotent for duplicate receiver submissions.

### Step 4: Event is stored

If the event is new, it is inserted into `button_events`.

The database also enforces the unique constraint, so the backend remains safe even if two duplicate requests race each other.

### Step 5: Notifications are triggered

After storing the event, the backend loads the user's push tokens and calls the configured notification sender.

Right now that sender only writes a trace log, but the call site is already in place for real push integration later.

## Device Management Flow

The backend currently supports a practical development lifecycle:

1. create a user with `POST /v1/users`
2. use the returned user id in `x-user-id`
3. create a receiver with `POST /v1/devices/receivers`
4. create a button paired to that receiver with `POST /v1/devices/buttons`
5. list buttons and receivers with `GET /v1/devices`

When creating a button, the backend verifies that the referenced receiver belongs to the authenticated user. That prevents one user from binding buttons to another user's receiver.

## Heartbeats

Receivers send `POST /v1/receivers/heartbeat` with their bearer token.

The backend updates `last_seen_at`, which makes it possible to:

- monitor receiver health
- detect offline devices
- surface receiver status in the app later

## Current HTTP API

### Health

- `GET /`
- `GET /health`

### User bootstrap

- `POST /v1/users`

### Device management

- `GET /v1/devices`
- `POST /v1/devices/receivers`
- `POST /v1/devices/buttons`
- `POST /v1/receivers/heartbeat`

### Events

- `GET /v1/events`
- `POST /v1/events/button-press`

### Push tokens

- `POST /v1/push-tokens`

## Local Development

### 1. Set environment variables

Use [backend/.env.example](/Users/anton/Dev/signal/backend/.env.example) as the starting point.

Required:

- `DATABASE_URL`

Optional:

- `SIGNAL_BACKEND_HOST`
- `SIGNAL_BACKEND_PORT`
- `DATABASE_MAX_CONNECTIONS`
- `CORS_ALLOW_ORIGIN`
- `RUST_LOG`

### 2. Start PostgreSQL

You can run Postgres with:

```sh
docker compose -f backend/docker-compose.yml up postgres
```

### 3. Run the backend

```sh
cargo run -p signal-backend
```

The backend runs migrations automatically during startup.

## Example API Walkthrough

### Create a user

```http
POST /v1/users
Content-Type: application/json

{
  "email": "nurse@example.com"
}
```

### Create a receiver

```http
POST /v1/devices/receivers
Content-Type: application/json
x-user-id: <user-uuid>

{
  "name": "Front desk receiver"
}
```

The response returns a receiver record plus its bearer auth token.

### Create a button

```http
POST /v1/devices/buttons
Content-Type: application/json
x-user-id: <user-uuid>

{
  "id": "button-123",
  "receiver_id": "<receiver-uuid>",
  "label": "Room 101"
}
```

### Register a push token

```http
POST /v1/push-tokens
Content-Type: application/json
x-user-id: <user-uuid>

{
  "platform": "ios",
  "token": "apns-or-fcm-token"
}
```

### Send a button press from the receiver

```http
POST /v1/events/button-press
Content-Type: application/json
Authorization: Bearer <receiver-auth-token>

{
  "button_id": "button-123",
  "event_counter": 123,
  "received_at": "2026-03-22T17:00:00Z"
}
```

### Read event history in the app

```http
GET /v1/events?limit=50
x-user-id: <user-uuid>
```

## Testing

The test suite currently focuses on the most important piece of business logic: event ingestion.

`tests/events_service.rs` verifies:

- a new button press creates an event
- notifications are triggered for new events
- duplicate `event_counter` submissions are deduplicated

The tests use an in-memory fake store rather than a real PostgreSQL database so the service logic can be tested in isolation.

## Known Gaps And Next Steps

The backend is a solid Phase 1 foundation, but these pieces are intentionally still open:

- replace `x-user-id` development auth with real user authentication
- integrate APNs and FCM in the notification service
- add receiver queue replay and backfill support
- add button pairing and provisioning flows
- add signature verification or shared-secret validation rules if the backend eventually validates button-origin authenticity
- add pagination and filtering for event history
- add integration tests against a real PostgreSQL instance

## Summary

The backend is organized around a clean split between transport, business logic, and persistence:

- routes handle HTTP
- services handle workflow logic
- the store trait handles persistence
- PostgreSQL stores the source of truth

That structure should make the next phases easier to extend without having to rewrite the ingestion path.
