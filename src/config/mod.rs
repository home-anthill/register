use std::env;
use std::fmt;

use dotenvy::dotenv;
use serde::Deserialize;
use tracing::info;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::writer::MakeWriterExt;

/// Which runtime environment the application is running in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Testing,
    Production,
}

impl AppEnv {
    /// Reads the `ENV` environment variable. Returns `Testing` only when the
    /// value is exactly `"testing"`; any other value (including absent) is
    /// treated as `Production`.
    pub fn from_env() -> Self {
        match env::var("ENV").as_deref() {
            Ok("testing") => Self::Testing,
            _ => Self::Production,
        }
    }

    pub fn is_testing(&self) -> bool {
        matches!(self, Self::Testing)
    }
}

#[derive(Deserialize)]
pub struct Env {
    pub log_level: Option<String>,
    pub mongo_uri: String,
    pub mongo_db_name: String,
    pub api_token_hash_secret: String,
    pub api_token_encryption_key: String,
    /// Maximum number of retry attempts after the first MongoDB connection try.
    /// Total attempts = mongo_max_retries + 1. Defaults to 50 when absent.
    pub mongo_max_retries: Option<u32>,
}

impl fmt::Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Env")
            .field("log_level", &self.log_level)
            .field("mongo_uri", &"[REDACTED]")
            .field("mongo_db_name", &self.mongo_db_name)
            .field("api_token_hash_secret", &"[REDACTED]")
            .field("api_token_encryption_key", &"[REDACTED]")
            .field("mongo_max_retries", &self.mongo_max_retries)
            .finish()
    }
}

pub fn init() -> (Env, AppEnv) {
    // Load the .env file
    dotenv().ok();
    let env = envy::from_env::<Env>().expect("failed to parse environment variables");
    let app_env = AppEnv::from_env();

    // Configure logging if not in test env.
    // We use set_global_default (not .init()) intentionally: .init() would also install
    // a LogTracer bridge for the `log` crate, which prevents Rocket from installing its
    // own RocketLogger. Without RocketLogger, Rocket's startup output (routes, config,
    // launched URL) is silently dropped. By skipping LogTracer, Rocket gets to install
    // its own logger and prints its startup info directly to stdout.
    if !app_env.is_testing() {
        let stdout_max_level =
            env.log_level.as_deref().and_then(|s| s.parse::<tracing::Level>().ok()).unwrap_or(tracing::Level::DEBUG);
        let stdout = std::io::stdout.with_filter(|meta| meta.target() == "app").with_max_level(stdout_max_level);
        let debug_file = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("info")
            .filename_suffix("log")
            .max_log_files(5)
            .build("./logs")
            .expect("initializing rolling info_file appender failed")
            .with_max_level(tracing::Level::INFO);
        let error_file = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("error")
            .filename_suffix("log")
            .max_log_files(5)
            .build("./logs")
            .expect("initializing rolling error_file appender failed")
            .with_filter(|meta| meta.target() == "app")
            .with_max_level(tracing::Level::ERROR);
        let writer = debug_file.and(error_file).and(stdout);
        let subscriber = tracing_subscriber::fmt()
            .compact()
            .with_writer(writer)
            .with_ansi(false)
            .with_max_level(tracing::Level::DEBUG)
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("Unable to install global subscriber");
    }

    info!(target: "app", "Starting application...");

    // Print .env vars
    print_env(&env);
    (env, app_env)
}

fn print_env(env: &Env) {
    info!(target: "app", "log_level = {}", env.log_level.as_deref().unwrap_or("debug"));
    info!(target: "app", "mongo_uri = [REDACTED]");
    info!(target: "app", "mongo_db_name = {}", env.mongo_db_name);
    info!(target: "app", "api_token_hash_secret = [REDACTED]");
    info!(target: "app", "api_token_encryption_key = [REDACTED]");
    info!(target: "app", "mongo_max_retries = {}", env.mongo_max_retries.unwrap_or(50));
}

#[cfg(test)]
mod tests {
    use super::Env;

    #[test]
    fn env_debug_redacts_mongo_uri() {
        let env = Env {
            log_level: Some("debug".to_string()),
            mongo_uri: "mongodb://user:password@localhost:27017/sensors".to_string(),
            mongo_db_name: "sensors".to_string(),
            api_token_hash_secret: "hash-secret".to_string(),
            api_token_encryption_key: "encryption-key".to_string(),
            mongo_max_retries: Some(3),
        };

        let debug = format!("{env:?}");

        assert!(debug.contains("mongo_uri: \"[REDACTED]\""));
        assert!(debug.contains("api_token_hash_secret: \"[REDACTED]\""));
        assert!(debug.contains("api_token_encryption_key: \"[REDACTED]\""));
        assert!(!debug.contains("password"));
        assert!(!debug.contains("mongodb://user"));
        assert!(!debug.contains("hash-secret"));
        assert!(!debug.contains("encryption-key"));
    }
}
