# Auth Service

Authentication service built with **Rust**, **Axum**, and **MongoDB or MySQL** (switch with **`DATABASE_DRIVER`**), optional **Redis** (`REDIS_URL`), user registration, login, **JWT (HS256)** issuance, **`GET /api/v1/auth/getme`** protected by **Bearer JWT validation middleware**, and API documentation via **OpenAPI / Swagger UI** ( **`bearer_auth`** security scheme so you can try tokens in Swagger).

## Requirements

| Component | Version / notes |
|-----------|-----------------|
| Rust | Edition **2024** (supported toolchain, e.g. 1.85+) |
| **API database** | **MongoDB** (`DATABASE_DRIVER=mongo`, `MONGODB_URI`) **or** **MySQL** (`DATABASE_DRIVER=mysql`, `DATABASE_URL`); SQL migrations in **`migrations/`** run automatically on API startup for MySQL |
| Environment variables | See `.env.example` — depends on **`APP_MODE`** (below) |
| **Worker mode** | System **librdkafka** (e.g. macOS: `brew install librdkafka`) — `rdkafka` links dynamically |
| **Cron mode** | No extra system libs; schedule via `CRON_SCHEDULE` |
| **Redis (API, optional)** | If **`REDIS_URL`** is set, the API opens an async **`ConnectionManager`** at startup and exposes **`AppState.redis`** for cache / rate limiting / sessions (handlers use `State<Arc<AppState>>`) |

### Run modes (`APP_MODE`)

| Value | Behavior | Required env (in addition to shared) |
|--------|-----------|-------------------------------------|
| **`api`** (default) | HTTP server (Axum + Swagger) | **`JWT_SECRET`** (≥ 32 chars); **`DATABASE_DRIVER=mongo`** (default): **`MONGODB_URI`**, optional **`DATABASE_NAME`**; **`mysql`**: **`DATABASE_URL`**; optional **`REDIS_URL`**, **`HOST`**, **`PORT`** |
| **`worker`** | Kafka **consumer** (logs messages; extend for your handlers) | `KAFKA_BROKERS`, `KAFKA_TOPIC`, `KAFKA_GROUP_ID` |
| **`cron`** | **`tokio-cron-scheduler`** job (default: every minute) | Optional `CRON_SCHEDULE` (six-field: `sec min hour day month weekday`) |

**Embedding / tests:** the library also exposes **`modes::run_worker_with_handler`** (custom [`MessageHandler`](src/modes/worker/handler.rs)) and **`modes::run_cron_with_task`** (custom [`CronTask`](src/modes/cron/task.rs)) so you can start consumers or schedulers without forking `main`.

```bash
APP_MODE=api cargo run
DATABASE_DRIVER=mysql DATABASE_URL='mysql://user:pass@127.0.0.1:3306/auth_service' APP_MODE=api cargo run
APP_MODE=worker KAFKA_BROKERS=localhost:9092 KAFKA_TOPIC=events KAFKA_GROUP_ID=auth-worker cargo run
APP_MODE=cron CRON_SCHEDULE='0 * * * * *' cargo run
```

### Main dependencies (crates)

- **axum** — HTTP server and routing  
- **tower** / **tower-http** — service composition and middleware (CORS, tracing)  
- **mongodb** + **bson** — user persistence (**`DATABASE_DRIVER=mongo`**)  
- **sqlx** (MySQL runtime + **migrate** + **macros**) — user persistence and schema migrations (**`DATABASE_DRIVER=mysql`**)  
- **argon2** — password hashing  
- **jsonwebtoken** — JWT encode/decode (signing + **`exp`** validation)  
- **utoipa** + **utoipa-swagger-ui** — OpenAPI spec and Swagger UI  
- **validator** — request body validation  
- **tokio** — async runtime  
- **redis** ( **`tokio-comp`**, **`connection-manager`**) — optional API cache / pub-sub (`REDIS_URL`)  
- **rdkafka** — Kafka consumer (**worker** mode; requires system librdkafka)  
- **tokio-cron-scheduler** — scheduled jobs (**cron** mode)  
- **futures** — `StreamExt` for the Kafka message stream  
- **async-trait** — async traits (`AuthService`, `UserRepository`, `MessageHandler`, `CronTask`)  

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `POST` | `/api/v1/auth/register` | Sign up + JWT |
| `POST` | `/api/v1/auth/login` | Login + JWT |
| `GET` | `/api/v1/auth/getme` | Current user profile; requires `Authorization: Bearer` plus the access token from login/register |
| `GET` | `/swagger-ui` | Swagger UI (see also `/api-docs/openapi.json`; use **Authorize** with the JWT) |

## Running locally

1. Copy `.env.example` to `.env` and adjust values.  
2. Start the database you configured (**MongoDB** or **MySQL**). For MySQL, create the empty database named in **`DATABASE_URL`** before the first run (migrations create the **`users`** table). If you set **`REDIS_URL`**, ensure Redis is reachable before starting the API.  
3. From the project root:

```bash
cargo run
```

Logging: set `RUST_LOG=info` (or `debug` for more detail).

**Environment variables:** follow **`.env.example`** — (1) `APP_MODE`, (2) API user storage (`DATABASE_DRIVER` + either Mongo **or** MySQL vars), (3) JWT / HTTP, optional Redis (`REDIS_URL`), then worker or cron blocks only if you use those modes.

## Module layout (overview)

```
migrations/         # SQL migrations (MySQL only; applied on API startup)
src/
  lib.rs            # Crate root: re-exports config, domain, http, modes, …
  main.rs           # Binary: LoadedApp::from_env() → modes::run_api | run_worker | run_cron
  config/
    mod.rs          # AppConfig (`DatabaseSettings`), LoadedApp re-exports
    env.rs          # load_api / load_worker / load_cron
    mode.rs         # AppMode, WorkerSettings, CronSettings, LoadedApp
  modes/
    api/
      mod.rs        # `run` (compose wiring + server)
      wiring.rs     # `DatabaseSettings` → `UserRepository`, `AuthService`, `AppState`, `routes::router`
      server.rs     # bind, logs, `axum::serve`
    worker/
      mod.rs        # Consumer loop + `run` / `run_with_handler`
      kafka.rs      # `StreamConsumer` factory from `WorkerSettings`
      handler.rs    # `InboundMessage`, `MessageHandler`, default `LogMessageHandler`
    cron/
      mod.rs        # `run` / `run_with_task`
      scheduler.rs  # start, Ctrl+C, shutdown
      task.rs       # `CronTask`, `LoggingCronTask`, job registration
  domain/
    mod.rs
    user.rs         # User entity
  error/
    mod.rs          # AppError + ErrorBody + IntoResponse + From impls
  http/
    mod.rs          # submodules + re-export ApiDoc
    routes.rs
    middleware/
      mod.rs          # re-exports + submodules
      auth.rs         # `require_jwt`, `AuthUser`
      request_log.rs  # JSON access log (`log_request`)
      redact.rs       # header/body redaction for logging
    openapi.rs
    schemas.rs
    handlers/
      mod.rs
      auth.rs
      health.rs
  jwt/
    mod.rs          # AccessClaims + encode/decode (service + middleware)
  repository/
    mod.rs
    user_repository.rs
    mongo_user_repository.rs
    mysql_user_repository.rs
  service/
    mod.rs
    auth.rs         # AuthServiceImpl + trait wiring
  state.rs          # AppState (auth, jwt_secret, optional redis::ConnectionManager)
```

**Separation:** **domain** is distinct from HTTP **DTOs** (`http/schemas.rs`); persistence sits behind the **`UserRepository`** trait so implementations can be swapped or mocked.

## Application flow (summary)

1. **`main`** calls **`LoadedApp::from_env()`** (`APP_MODE` → validate only the variables for that mode), then **`match`**es on **`LoadedApp::Api` / `Worker` / `Cron`** and runs **`modes::run_api`**, **`run_worker`**, or **`run_cron`**.  
2. **API mode** (`modes/api/`): **`wiring::build_router`** matches **`config.database`** (**`DatabaseSettings::Mongo`** → Mongo client + **`MongoUserRepository`**; **`Mysql`** → pool + migrations + **`MysqlUserRepository`**), builds **`AuthServiceImpl`** over **`Arc<dyn UserRepository>`**, **`AppState`**, and **`routes::router`**; **`server::listen_and_serve`** runs **`axum::serve`**. **`routes::router`** merges Swagger with API routes, applies **layers** (middleware), then **`.with_state(state)`** once at the end (Axum 0.8: `Router<Arc<AppState>>` → `Router<()>` for `serve`). **Worker** (`modes/worker/`) subscribes via **`kafka::stream_consumer`** and runs the poll loop (**`run_with_handler`** for custom processing). **Cron** (`modes/cron/`) registers jobs and runs **`scheduler::run_until_ctrl_c`** (**`run_with_task`** for custom jobs).  
3. **(API)** Register/login handlers extract `State<Arc<AppState>>` and `Json<...>`, delegating to **`auth.register` / `auth.login`**. The **`get_me`** handler uses **`Extension<AuthUser>`** populated by the JWT middleware.  
4. **(API)** **Service:** validation → Argon2 password hash/verify → read/write users via repository → issue JWT via **`jwt::encode_access_token`** (`sub` = hex `ObjectId`, `exp` / `iat`); **`get_me`** loads the user from the DB via **`find_by_id`**.  
5. **(API)** **Persistence:** **Mongo** — `users` collection, unique index on `email`, lookup by `_id`. **MySQL** — `users` table (24-char hex `id` matching **`ObjectId`** / JWT `sub`), unique `email`; **`sqlx::migrate!`** on startup.  
6. **(API)** Domain errors become HTTP responses via **`IntoResponse`** on **`error::AppError`** (`error/mod.rs`).

## Data processing

- **Register**  
  - Normalize email (trim, lowercase).  
  - Validate password (minimum length, etc. via `validator`).  
  - Check for duplicate email; hash password with **Argon2**; create user document (`ObjectId`, `created_at`); persist to MongoDB; return JWT + public user projection (no password hash).  

- **Login**  
  - Find user by email; verify password against stored hash; return JWT + public user.  

- **Get me (`GET /api/v1/auth/getme`)**  
  - Middleware **`require_jwt`** reads `Authorization: Bearer …`, verifies the JWT with **`jwt_secret`** on `AppState` (same secret used to sign tokens), parses `sub` into `ObjectId`, inserts **`AuthUser`** into request extensions.  
  - Handler calls **`auth.get_me(user_id)`** → **`find_by_id`** in MongoDB → **`UserPublic`** (no password hash).  

- **JWT**  
  - Algorithm **HS256**; secret from `JWT_SECRET`; minimal claims: `sub` (user id), `iat`, `exp`. Verification uses **`jsonwebtoken::Validation::default()`** (including **`exp`**).  

- **Not stored in the DB**  
  - Plain-text passwords; only Argon2 hashes.

## Middleware (current implementation)

Middleware is implemented as **Tower layers** / **Axum `from_fn` / `from_fn_with_state`** in `src/http/routes.rs` and `src/http/middleware/`:

1. **`log_request`** — root **`from_fn`**: emits **one JSON object per line** (target `http_request`) shaped like `{"http_request":{"method","path","status","latency_ms","request_headers","request_body","response_headers","response_body"}}`. **`request_headers`** / **`response_headers`** are JSON objects with **lowercase** keys; sensitive header names map to **`"<redacted>"`**. **`request_body`** / **`response_body`** are parsed JSON when `Content-Type` is JSON (recursive redaction for keys such as **`password`**, **`access_token`**, **`token`**, …); empty body → **`null`**; non-JSON → string. Buffers up to **256 KiB** per direction (**`413`** if the request exceeds that). **`/swagger-ui`** and **`/api-docs/*`** log **`"response_body":"<omitted>"`** (string) to skip huge OpenAPI payloads. With the default **`tracing_subscriber::fmt`** layer, the line is usually prefixed by timestamp/level (use a JSON subscriber or `fmt::layer().json()` if you need a bare JSON-only line).  
2. **`CorsLayer`** (`tower-http::cors`) — on the API sub-router only: permissive CORS (`Any`) for origin/method/header (tighten in production).  
3. **`require_jwt`** — only on **`GET /api/v1/auth/getme`**, via **`route_layer(middleware::from_fn_with_state(state.clone(), require_jwt))`**. It receives **`State<Arc<AppState>>`** (explicit layer state), verifies the Bearer token with **`app.jwt_secret`**, then stores **`AuthUser { user_id }`** in **request extensions** for **`Extension<AuthUser>`** in the handler.

**`log_request`** runs for **all** merged routes (Swagger UI, OpenAPI JSON, `/health`, `/api/v1/auth/*`). CORS still applies only to the API router subtree.

To add more protected routes, reuse the same pattern: **`get(...).route_layer(from_fn_with_state(..., require_jwt))`** or refactor into a **nested `Router`** with a shared `route_layer`.

## Sequence diagrams (Mermaid)

### Register

```mermaid
sequenceDiagram
    autonumber
    participant C as Client
    participant A as Axum (Router + layers)
    participant H as Handler register
    participant S as AuthService
    participant R as UserRepository (Mongo)
    participant D as MongoDB

    C->>A: POST /api/v1/auth/register (JSON)
    A->>A: CORS + Trace
    A->>H: State + Json body
    H->>S: register(RegisterRequest)
    S->>S: validate, normalize email
    S->>R: find_by_email
    R->>D: findOne { email }
    D-->>R: document / null
    R-->>S: Option<User>
    alt email already exists
        S-->>H: Conflict
        H-->>C: 409 + ErrorBody
    else new email
        S->>S: Argon2 hash password
        S->>R: create(User)
        R->>D: insertOne
        D-->>R: ok
        S->>S: encode JWT (HS256)
        S-->>H: AuthResponse
        H-->>C: 200 + JSON (token + public user)
    end
```

### Login

```mermaid
sequenceDiagram
    autonumber
    participant C as Client
    participant A as Axum (Router + layers)
    participant H as Handler login
    participant S as AuthService
    participant R as UserRepository (Mongo)
    participant D as MongoDB

    C->>A: POST /api/v1/auth/login (JSON)
    A->>A: CORS + Trace
    A->>H: State + Json body
    H->>S: login(LoginRequest)
    S->>S: validate, normalize email
    S->>R: find_by_email
    R->>D: findOne { email }
    D-->>R: user document
    R-->>S: User (with password_hash)
    alt user missing or wrong password
        S-->>H: Unauthorized
        H-->>C: 401 + ErrorBody
    else valid credentials
        S->>S: Argon2 verify + JWT encode
        S-->>H: AuthResponse
        H-->>C: 200 + JSON
    end
```

### High-level request path (server running)

```mermaid
sequenceDiagram
    participant L as TcpListener
    participant X as axum::serve
    participant R as Router
    participant M as Middleware stack
    participant H as Handler

    L->>X: incoming connection
    X->>R: dispatch request
    R->>M: layer chain (order follows .layer placement)
    M->>H: matching path + method
    H-->>M: Response
    M-->>R: Response
    R-->>X: Response
    X-->>L: to client
```

### Get me (JWT required)

```mermaid
sequenceDiagram
    autonumber
    participant C as Client
    participant A as Axum (CORS + Trace)
    participant MW as require_jwt middleware
    participant H as Handler get_me
    participant S as AuthService
    participant R as UserRepository
    participant D as MongoDB

    C->>A: GET /api/v1/auth/getme + Authorization: Bearer JWT
    A->>A: CORS + Trace
    A->>MW: Request
    MW->>MW: decode JWT, check exp, parse sub → ObjectId
    alt invalid / missing token
        MW-->>C: 401 + ErrorBody
    else valid token
        MW->>MW: extensions.insert(AuthUser)
        MW->>H: State + Extension(AuthUser)
        H->>S: get_me(user_id)
        S->>R: find_by_id
        R->>D: findOne { _id }
        D-->>R: document / null
        alt user not found
            S-->>H: Unauthorized
            H-->>C: 401 + ErrorBody
        else ok
            S-->>H: UserPublic
            H-->>C: 200 + JSON
        end
    end
```

## Extending: another service (e.g. Payment) that only accepts JWT

In **this** service, **`/api/v1/auth/getme`** already validates **Bearer JWT** as described above. You can reuse the same pattern in a separate **payment service**.

For a **payment service** (or another microservice) that only processes requests with a login token:

1. **Shared secret contract**  
   Use the **same `JWT_SECRET`** as the auth service (or a public key if you move to RS256 + JWKS). With HS256, every verifier must trust the same secret, or you centralize verification at an API gateway.

2. **Flow in the payment service**  
   - Read the `Authorization: Bearer <token>` header.  
   - Decode and verify the signature (`jsonwebtoken::decode` + `DecodingKey` from the same secret).  
   - Enforce `exp` / `iat`; use `sub` as **user id** for charging, idempotency, auditing.  
   - Return **401** if the token is missing, invalid, or expired.

3. **In Axum**  
   - Mirror **`http/middleware/auth.rs`** in this repo: **`from_fn_with_state` + `jsonwebtoken` verification + `extensions.insert(...)`**, or use a custom extractor.  
   - Apply the layer only to routes that require auth (e.g. `/payments/...`).

4. **No per-request call to the auth service**  
   JWT verification is **local** (cryptographic) as long as secrets/keys align; optional refresh-token denylist / logout may need Redis or extra introspection.

```mermaid
sequenceDiagram
    participant C as Client
    participant P as Payment service
    participant MW as JWT middleware / extractor
    participant PG as Payment use-case

    C->>P: POST /payments (Authorization: Bearer JWT)
    P->>MW: verify signature + exp
    alt invalid token
        MW-->>C: 401
    else valid
        MW->>PG: user id from sub (claims)
        PG-->>C: 200 + business result
    end
```

## Third-party integration

### Payment gateway (HTTP)

- Put HTTP clients (e.g. `reqwest`) in an **infrastructure** module (e.g. `infra::stripe` / `infra::xendit`), not directly in handlers.  
- The payment **service** calls a `PaymentGateway` **trait** you own; concrete types call the provider API with keys from **environment** / a secret manager.  
- Use **timeouts**, **bounded retries**, and **idempotency keys** (header or field) to avoid duplicate charges on network failures.  
- For provider webhooks, verify signatures/timestamps per their docs, then update DB state or publish an event (see brokers below).

### Message broker (Kafka, RabbitMQ, NATS, etc.)

- **Producer:** after important domain events (e.g. `UserRegistered`, `PaymentCompleted`), publish from the **service** or transaction **after** a successful DB commit for consistency.  
- **Consumer:** separate binary/worker or in-process task; subscribe to topics; idempotency via **message id** or DB deduplication.  
- Avoid blocking HTTP responses for long async work; for async-style flows, accept the request → write outbox / enqueue → return **202** or a “pending” status if the business allows.

## Adding features in this repo

| Goal | Where to change |
|------|-----------------|
| New routes & OpenAPI | `http/handlers/` (`auth.rs`, `health.rs`), `http/routes.rs`, `http/openapi.rs` |
| New business logic | `service/` (+ trait if you need swappable implementations) |
| New data access | `repository/` (`UserRepository` + `mongo_*` / `mysql_*`) |
| MySQL schema changes | Add a versioned file under **`migrations/`** (same embedded migrator as startup) |
| Switch DB driver | `.env` **`DATABASE_DRIVER`** + **`config/env.rs`** / **`DatabaseSettings`** |
| New entities | `domain/` |
| API request/response shapes | `http/schemas.rs` |
| Global / per-route middleware | `http/routes.rs` — `.layer` or `route_layer(from_fn_with_state(...))`; logic in `http/middleware/` |
| API error type + JSON error body (`ErrorBody`) | `error/mod.rs` |
| JWT encode/decode | `jwt/mod.rs` (could be a shared crate for other services) |
| Optional Redis in API | `.env` **`REDIS_URL`**; wiring in `modes/api/wiring.rs`; use `state.redis` in handlers |
| Kafka consumer wiring / message handling | `modes/worker/kafka.rs`, `modes/worker/handler.rs`, `modes/worker/mod.rs` |
| Cron job logic / schedule lifecycle | `modes/cron/task.rs`, `modes/cron/scheduler.rs`, `modes/cron/mod.rs` |
| New configuration / mode | `config/mod.rs`, `config/env.rs`, `config/mode.rs`, `modes/`, `.env.example` |

### Crate layout

The package is a **library** (`lib.rs`) plus a **binary** (`main.rs`). Integration tests or other binaries can `use auth_service::modes::…` and call **`run_api`**, **`run_worker`**, **`run_worker_with_handler`**, **`run_cron`**, or **`run_cron_with_task`** with config loaded however you prefer (not only `LoadedApp::from_env()`).

---

This document reflects the project at the time of writing; always verify against the code under `src/` when things change.
