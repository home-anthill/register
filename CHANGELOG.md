# Changelog

## Unreleased

### Features

- Added idempotent `DELETE /sensors/<deviceUuid>/features/<featureUuid>` cleanup for
  removing registered sensor documents when a device is deleted upstream.

## 3.0.1

### Tests

- Added unit tests for registration input validation, covering valid payloads, UUID v4 checks, malformed MAC addresses, empty or oversized text fields, non-ASCII text, and control characters.
- Added unit tests for sensor response building, covering float and integer value extraction plus internal-error responses for wrong BSON value types and missing timestamps.


## 3.0.0

### Features

- Added `FeatureName` with `is_float()`, `as_str()`, `Display`, `FromStr`, and `ALL`.
- Added generic `Sensor<V>` and `RegisterInput::into_sensor_bson`.
- Added `pub COLLECTION_NAME` for the `sensors` collection.
- Added `LOG_LEVEL` for configurable stdout logging.
- Added `Makefile check` target for `cargo audit`.

### Bug fixes

- Fixed `Env` debug redaction so `mongo_uri` is never exposed.
- Duplicate MongoDB key errors now return HTTP 409 Conflict.
- Integer and float sensor values now return native JSON numbers without lossy casts.
- `AppEnv::from_env()` is called once at startup and passed explicitly to `db::connect`.
- `ApiError::respond_to` now emits both `message` and `code`.

### Security issues

- Redacted MongoDB URI credentials and sensitive query parameters in logs.
- Centralized `ENV` reading through `AppEnv` to remove the startup TOCTOU window.
- Replaced connection panics with graceful aborts that do not expose driver details.
- Configured MongoDB `connect_timeout = 10s` and `server_selection_timeout = 30s`.
- Created a compound unique index on `(deviceUuid, featureUuid, featureName)`.
- Removed DB-layer `.unwrap()` calls and kept driver details in internal error logs only.
- Used `Display` logging instead of `Debug` logging at all log sites.
- Hardened input validation for ObjectIds, UUID v4 values, MAC addresses, and bounded ASCII strings.
- Stored sensor API tokens as `apiTokenHash` plus AES-GCM `apiTokenEncrypted`; required hash and encryption secrets with no fallbacks.
- Rejected `API_TOKEN_HASH_SECRET` values shorter than 32 characters at startup.
- Normalized MAC addresses to uppercase on insert.
- Validated GET path parameters before database access and removed user UUIDs from `info!` logs.
- Set the JSON body limit to 8 KiB.
- Hardened the runtime image with `dhi.io/debian-base:trixie`, UID 65534, isolated CA cert installation, and pre-owned `/app/logs`.

### Idiomatic Rust issues

- Introduced typed `ValidationError` with `thiserror`.
- Replaced manual error implementations with `thiserror::Error`.
- Changed HTTP 400/404 logs to `warn!`; 500/503 remain `error!`.
- Used `let...else` for route-handler early returns.
- Made `keep_alive` synchronous.
- Removed redundant `else` branches after return guards.
- Simplified delay arithmetic.
- Replaced `as_document()?.clone()` with `Bson::Document(doc)` pattern matching.
- Used `InvalidFeatureName` as a unit struct for `FromStr::Err`.
- Reworked `is_valid_mac` to avoid `Vec` allocation.
- Replaced wildcard imports in tests with explicit imports.

### Chores

- Extracted `build_sensor_response` and shared error helpers.
- Made `validate_uuid_field` public and removed its wrapper.
- Updated Dockerfile to a 5-stage build with `--no-install-recommends` and an absolute `ENTRYPOINT`.
- Removed unused `futures` dependency.
- Added `thiserror`.
- Bumped `mongodb` and `tracing-subscriber`.
- Bumped package version from `2.0.1` to `2.1.0`.

### Tests

- Updated CI to run tests against a MongoDB 8.0 replica set with coverage reporting.
