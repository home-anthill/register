# Changelog

## Security

- MongoDB URI credentials and sensitive query params redacted in all logs; custom `Debug` impl for `Env` also redacts `mongo_uri`
- `AppEnv` enum centralises `ENV` var reading; `from_env()` called exactly once at startup (eliminates TOCTOU window)
- Connection failure uses graceful abort instead of `panic!`; panic messages never contain error details
- MongoDB client configured with `connect_timeout = 10s` and `server_selection_timeout = 30s` to prevent indefinite hangs
- Compound unique index `(deviceUuid, featureUuid, featureName)` created at startup; conflict detected via structured error code (not string matching)
- All `.unwrap()` calls removed from DB layer; driver details only logged internally at `error!` level, never surfaced to callers
- `ApiError::respond_to` emits full `{message, code}` JSON (previously dropped the `code` field)
- All log sites use `{}` (Display) instead of `{:?}` (Debug) to avoid leaking driver internals
- `validate()` enforces ObjectId format for `profileOwnerId`, UUID v4 for `deviceUuid`/`featureUuid`/`apiToken`, `XX:XX:XX:XX:XX:XX` MAC format, and bounded non-empty ASCII strings for `model`/`manufacturer`
- MAC address normalized to uppercase on insert
- GET endpoint validates all path parameters before touching the database; user-supplied UUIDs removed from `info!`-level logs
- JSON body limit set to 8 KiB
- Hardened runtime image (`dhi.io/debian-base:trixie`), non-root user (UID 65534), `system-deps` stage isolates CA cert installation

## Bug Fixes

- `DbError` converted to enum with `AlreadyExists` / `Other(String)` variants; duplicate-key inserts (MongoDB error code 11000) now return HTTP 409 Conflict instead of 400/500
- Integer sensor values returned as native JSON integers; float sensors as native JSON floats — eliminates silent precision loss from `i64 as f64` cast
- `AppEnv::from_env()` invoked exactly once at startup; `db::connect` receives explicit `app_env` parameter instead of re-reading `ENV`

## Refactoring

- `FeatureName` enum extracted to its own module with `is_float()`, `as_str()`, `Display`, `FromStr`, and `ALL` constant; replaces earlier `SensorType` naming
- Typed `ValidationError` introduced via `thiserror`
- Generic `Sensor<V>` replaces duplicate `IntSensor`/`FloatSensor` structs; `RegisterInput::into_sensor_bson` inherent method replaces free function
- `pub COLLECTION_NAME` constant introduced; all hardcoded `"sensors"` strings replaced
- `build_sensor_response` extracted; `bad_request` / `internal_error` helpers eliminate repeated `json!` construction
- `validate_uuid_field` made public and called directly; trivial wrapper removed
- `LOG_LEVEL` env var for configurable stdout log level; `set_global_default` used instead of `.init()` to preserve Rocket's own logger

## Idiomatic Rust

- HTTP 400/404 log level changed from `error!` to `warn!`; 500/503 remain `error!`
- `let...else` used for early-return-on-error in route handlers; `keep_alive` made synchronous (no `.await`)
- Redundant `else` branches removed after `return` guards; delay arithmetic simplified
- `Bson::Document(doc)` pattern match replaces `as_document()?.clone()`
- `InvalidFeatureName` unit struct used as `FromStr::Err` instead of `String`
- `is_valid_mac` uses counted iterator — no `Vec` allocation
- `thiserror::Error` derive replaces manual `Display` / `Error` impls
- Wildcard imports replaced with explicit `use` items in tests

## Infrastructure

- Dockerfile uses 5-stage multi-stage build (chef → planner → builder → system-deps → runtime); `--no-install-recommends`; `/app/logs` pre-created owned by UID 65534; absolute `ENTRYPOINT`
- `Makefile`: `check` target added for `cargo audit`
- `Cargo.toml`: removed unused `futures` dependency; `thiserror` added; `mongodb` and `tracing-subscriber` bumped; package version `2.0.1` → `2.1.0`
- CI test job runs against MongoDB 8.0 replica set with coverage reporting
