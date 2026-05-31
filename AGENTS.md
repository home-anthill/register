# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## Project Overview

**ks89-register** is a Rust microservice (Rocket framework) for IoT sensor registration and data retrieval, part of the home-anthill ecosystem. It uses MongoDB for storage and runs in Docker.

## Common Commands

All commands are in the Makefile:

- `make build` — format, lint, then `cargo build`
- `make release` — format, lint, then `cargo build --release`
- `make run` — watch mode with auto-reload (requires cargo-watch)
- `make test` — run all tests (sets `ENV=testing`, uses real MongoDB)
- `make test-coverage` — generate coverage report with grcov to `coverage/html`
- `make fmt` — format with rustfmt
- `make lint` — run clippy
- `make deps` — install all dev tools (clippy, rustfmt, cargo-watch, grcov)
- `make clean` — clean build artifacts

Running a single test: `ENV=testing cargo test <test_name> -- --nocapture`

Tests require a running MongoDB instance (see `.env_template` for `MONGO_URI`). All integration tests must run single-threaded: `ENV=testing cargo test -- --nocapture --test-threads 1`.

## Architecture

**Entry point:** `src/main.rs` launches the Rocket server via Rocket's `#[rocket::launch]` macro, attaches the DB connection handler, mounts API routes, and registers error catchers.

**API routes** (`src/routes/api.rs`):
- `POST /sensors/register/<feature_name>` — register a sensor (feature names: temperature, humidity, light, motion, airquality, airpressure, online). Request body is JSON `RegisterInput`; route validates input, checks for duplicates (409 Conflict if exists), inserts with `value = 0`, then responds with the inserted doc. Returns 400 Bad Request if JSON is malformed or body exceeds 8 KiB limit.
- `GET /sensors/<device_uuid>/features/<feature_uuid>/<feature_name>` — retrieve sensor value by device, feature UUID, and name. Returns 404 Not Found if sensor not found; returns JSON with integer or float `value` depending on feature type.
- `GET /keepalive` — health check endpoint, responds with 200 OK.

**Key modules:**
- `models/feature_name.rs` — `FeatureName` enum (temperature, humidity, light, motion, airquality, airpressure, online) with `is_float()` method (determines whether values are f64 or i64), `as_str()`, `Display`, `FromStr` (returning `InvalidFeatureName` error), and `ALL` constant slice; `InvalidFeatureName` is a unit struct error type via `thiserror`.
- `models/sensor.rs` — generic `Sensor<V>` struct with fields `apiTokenHash`, `apiTokenEncrypted`, `deviceUuid`, `featureUuid`, `featureName`, `value`, `createdAt`, `modifiedAt`; type aliases `IntSensor = Sensor<i64>` and `FloatSensor = Sensor<f64>`; `RegisterInput::into_sensor_bson()` converts request to BSON; `SensorError` typed error.
- `models/inputs.rs` — `RegisterInput` struct with snake_case JSON fields renamed to camelCase (`#[serde(rename_all = "camelCase")]`); `validate()` method (called by route handlers before DB operations); `validate_uuid_field()` public fn that requires UUID v4 format; `profileOwnerId` validated via `ObjectId::from_str()`. Validation errors return 400 with JSON body.
- `db/mod.rs` — MongoDB connection initialization and lifecycle (`init()` attaches DB handler to Rocket). Ensures the unique compound index `(deviceUuid, featureUuid, featureName)` on startup (if conflict code 86 occurs, drops and recreates). Shared `pub COLLECTION_NAME` constant used by both production code and test helpers.
- `db/sensor.rs` — database operations: `insert()` (checks for existing doc via error code 11000, converts to `DbError::AlreadyExists`), `find_by_uuid()` (returns `Option<Document>`, routes extract and cast `value` as i64 or f64 based on feature type).
- `errors/api_error.rs` — `ApiResponse` struct with `message` and `code` fields; `ApiError` responder that implements `Responder` to emit JSON errors. Error responses include HTTP status and JSON body.
- `errors/db_error.rs` — `DbError` enum (`AlreadyExists` / `Other(String)`) via `thiserror`; duplicate-key MongoDB errors (code 11000) map to `AlreadyExists` (HTTP 409); other DB errors are `Other`. `DbError::other(msg)` is the constructor.
- `errors/validation_error.rs` — `ValidationError` with `thiserror::Error`, returned as HTTP 400 with JSON body.
- `catchers/` — HTTP error catchers: 400 Bad Request → `warn!`, 404 Not Found → `warn!`, 500 Internal Server Error → `error!`, 503 Service Unavailable → `error!`. Each logs the status and returns JSON error response.
- `config/` — environment initialization: `init()` reads `ENV`, `LOG_LEVEL`, `MONGO_URI`, `MONGO_DB_NAME`, `MONGO_MAX_RETRIES` via `dotenvy`/`envy`; returns `(Env, AppEnv)` tuple so `ENV` is read exactly once at startup; `AppEnv` is `Copy + Clone` (Testing/Production); sets up tracing subscriber with file + console output (production only) while preserving Rocket's own logger.
- `tests_integration/` — integration tests against real MongoDB. Helper modules: `db_utils` provides test DB initialization and sensor fixture insertion; `test_utils` provides common test setup; `register`, `keepalive`, `errors_catchers` contain test cases.

## Data Flow & Error Responses

**Sensor registration flow:**
1. Client sends `POST /sensors/register/temperature` with JSON body (deviceUuid, featureUuid, profileOwnerId, apiToken, macAddress)
2. Route handler deserializes to `RegisterInput` (400 if invalid JSON/schema)
3. Route calls `input.validate()` (400 with validation errors if UUIDs invalid, ObjectId parse fails, etc.)
4. Route calls `db::insert(input.into_sensor_bson())` (409 if duplicate key, 500 on other DB errors)
5. Success: returns 200 with inserted document JSON (includes `value: 0`, ISO timestamps)

**Error response format:**
All errors return JSON `{message: string, code: number}`. Examples:
- `POST /sensors/register/temperature` with bad UUID: `400 {message: "Invalid UUID format", code: 400}`
- `POST /sensors/register/temperature` with duplicate key: `409 {message: "Sensor already registered", code: 409}`
- `GET /sensors/unknown-uuid/features/.../temperature`: `404 {message: "Sensor not found", code: 404}`
- Unhandled server error: `500 {message: "Internal server error", code: 500}`

**MongoDB behavior:**
- New sensors inserted with `value = 0` (the `consumer` service updates value + timestamps later)
- Compound unique index `(deviceUuid, featureUuid, featureName)` prevents duplicates; on conflict, route returns HTTP 409
- Index is auto-created/verified at startup; if conflict (code 86), existing index is dropped and recreated
- Connection retries: `MONGO_MAX_RETRIES` env var controls retries after the first attempt (default: 50 retries = 51 total attempts)

## Code Conventions

- Rust 2024 edition
- Formatting: rustfmt with 120 char max width, 4-space indent (`rustfmt.toml`)
- JSON fields use camelCase (`#[serde(rename_all = "camelCase")]`)
- Logging uses the `tracing` crate with target `"app"`
- All async endpoints; MongoDB connection retries configurable via `MONGO_MAX_RETRIES` env var (default: 50 retries after the first attempt; total attempts = `MONGO_MAX_RETRIES + 1`)
- No `unwrap()` in production code — use `?` with `map_err`, `ok_or_else`, or `expect` with descriptive messages
- Prefer `&str` over `&String` in function parameters
- Error logging uses `{}` (Display), never `{:?}` (Debug), to avoid leaking internal driver details; MongoDB operation errors are logged at `error!` level
- `Env` has a custom `Debug` implementation and must keep `mongo_uri` redacted as `[REDACTED]`; do not derive `Debug` for it without preserving redaction.
- `apiToken` from registration requests is never stored plaintext. Sensor documents store `apiTokenHash` (HMAC-SHA-256 with `API_TOKEN_HASH_SECRET`) for lookup and `apiTokenEncrypted` (AES-GCM with `API_TOKEN_ENCRYPTION_KEY`) so consumers can verify signed MQTT payloads.
- MAC addresses are normalized to **uppercase** on insert (`to_ascii_uppercase()`).
- New sensors are always inserted with `value = 0` (type default). The `consumer` service updates the `value`, `createdAt`, and `modifiedAt` fields later.
- Float sensors (f64): `temperature`, `humidity`, `light`, `airpressure`. Integer sensors (i64): `motion`, `airquality`, `online`. This is governed by `FeatureName::is_float()`.

## Configuration

- `Rocket.toml` — server config (debug mode: port 8000, release mode: port 80); JSON body limit set to 8 KiB for both profiles
- `.env_template` — template file (gitignored `.env` is copied from this). Required vars: `MONGO_URI` (e.g. `mongodb://localhost:27017`), `MONGO_DB_NAME` (database name, ignored in test mode), `API_TOKEN_HASH_SECRET`, `API_TOKEN_ENCRYPTION_KEY`. Optional: `LOG_LEVEL` (debug/info/warn/error, default: debug), `MONGO_MAX_RETRIES` (integer retries after first attempt, default: 50)
- `config/mod.rs` — `init()` function reads env vars exactly once at startup and returns `(Env, AppEnv)` tuple. `AppEnv` is `Copy + Clone` so it's cheap to pass around. The function also sets up the tracing subscriber (file logging in production, console in tests)
- **Test database isolation**: when `ENV=testing`, the service ignores `MONGO_DB_NAME` and uses hardcoded `sensors_test` database. This prevents test runs from corrupting production data. Integration tests assume a real MongoDB instance is running (no mocking).
- **Logging behavior**:
  - **Production** (`ENV` not set or `Production`): writes rolling daily files to `./logs/` — `info.*.log` (all events, max 5 files) and `error.*.log` (errors only, max 5 files) — plus console output filtered to target `"app"` (filtering respects `LOG_LEVEL`)
  - **Testing** (`ENV=testing`): does not install the tracing subscriber; relies on Rocket's built-in logger

## Security Notes

- API token secrets are mandatory: `API_TOKEN_HASH_SECRET` must match services that query by `apiTokenHash` and must be at least 32 characters, and `API_TOKEN_ENCRYPTION_KEY` must match services that decrypt `apiTokenEncrypted`.
- Duplicate sensor registration returns HTTP 409 Conflict (mapped from `DbError::AlreadyExists`); logged at `warn!` level.
- MongoDB client is configured with `connect_timeout = 10s` and `server_selection_timeout = 30s` to prevent indefinite hangs.
- `GET /sensors/.../features/.../...` returns `value` as native JSON integer for int sensors and native float for float sensors; no `i64 as f64` cast.
- Panic messages on MongoDB connection failure do not include error details (already logged separately at `error!` level).

## CI/CD

**Dockerfile:** 5-stage multi-stage build (chef → planner → builder → system-deps → runtime). The `system-deps` stage installs CA certificates and pre-creates `/app/logs` owned by nobody (uid 65534). The runtime stage uses the hardened image `dhi.io/debian-base:trixie` (no package manager), runs as non-root uid 65534, and uses an absolute `ENTRYPOINT ["/app/register"]`.

When making significant changes (new features, bug fixes, refactors), append an entry to `CHANGELOG.md` in this directory.

GitHub Actions (`.github/workflows/docker-image.yml`):
- Test job: starts MongoDB 8.0 replica set, runs `make test-coverage`
- Build job: Docker multi-stage build, pushes to Docker Hub (`ks89/register`) on master/develop/tags
