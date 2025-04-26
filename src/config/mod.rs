use std::env;

use dotenvy::dotenv;
use serde::Deserialize;
use tracing::info;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[derive(Deserialize, Debug)]
pub struct Env {
    pub mongo_uri: String,
    pub mongo_db_name: String,
}

pub fn init() -> Env {
    // Load the .env file
    dotenv().ok();
    let env = envy::from_env::<Env>().ok().unwrap();

    // Configure logging
    if env::var("ENV") != Ok("testing".to_string()) {
        let stdout = std::io::stdout;
        let debug_file = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("all")
            .filename_suffix("log")
            .max_log_files(5)
            .build("./logs")
            .expect("initializing rolling debug_file appender failed")
            .with_filter(|meta| meta.target() == "app");
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
        tracing_subscriber::fmt()
            .compact()
            .with_writer(writer)
            .with_ansi(false)
            .init();
    }

    info!(target: "app", "Starting application...");

    // Print .env vars
    print_env(&env);
    env
}

fn print_env(env: &Env) {
    let mongo_uri = env.mongo_uri.clone();
    let mongo_db_name = env.mongo_db_name.clone();
    info!(target: "app", "env = {:?}", env);
    info!(target: "app", "mongo_uri = {}", mongo_uri);
    info!(target: "app", "mongo_db_name = {}", mongo_db_name);
}
