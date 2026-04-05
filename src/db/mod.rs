use std::future::Future;
use std::time::Duration;

use mongodb::bson::doc;
use mongodb::error::ErrorKind;
use mongodb::options::{ClientOptions, IndexOptions, ServerApi, ServerApiVersion};
use mongodb::{Client, Database, IndexModel};
use rocket::fairing::AdHoc;
use rocket::tokio::time::sleep;
use tracing::{error, info, warn};

use crate::config::{AppEnv, Env};

pub mod sensor;

pub const COLLECTION_NAME: &str = "sensors";

pub fn init(env_config: Env, app_env: AppEnv) -> AdHoc {
    AdHoc::try_on_ignite("Connecting to MongoDB", move |rocket| async move {
        match connect(env_config, app_env).await {
            Ok(database) => Ok(rocket.manage(database)),
            Err(error) => {
                error!(target: "app", "MongoDB - cannot connect: {}", error);
                Err(rocket)
            }
        }
    })
}

async fn connect(env_config: Env, app_env: AppEnv) -> mongodb::error::Result<Database> {
    let max_retries = env_config.mongo_max_retries.unwrap_or(50);
    let mongo_db_name = if app_env == AppEnv::Testing {
        error!(target: "app", "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        error!(target: "app", "!!! WARNING: ENV=testing — using database 'sensors_test' !!!");
        error!(target: "app", "!!! ALL PRODUCTION DATA WILL BE IGNORED                  !!!");
        error!(target: "app", "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        "sensors_test".into()
    } else {
        env_config.mongo_db_name
    };

    let mut client_options = ClientOptions::parse(&env_config.mongo_uri).await?;
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Set app_name
    client_options.app_name = Some("register".into());
    // Prevent indefinite hangs on unresponsive server
    client_options.server_selection_timeout = Some(Duration::from_secs(30));
    client_options.connect_timeout = Some(Duration::from_secs(10));

    // Create a new client and connect to the server
    let client = Client::with_options(client_options)?;
    let database = client.database(&mongo_db_name);

    info!(target: "app", "Pinging MongoDB server...");
    retry_connect_mongodb(|| async { database.run_command(doc! { "ping": 1 }).await }, max_retries).await?;

    ensure_indexes(&database).await?;

    Ok(database)
}

async fn ensure_indexes(database: &Database) -> mongodb::error::Result<()> {
    let collection = database.collection::<mongodb::bson::Document>(COLLECTION_NAME);
    let index = IndexModel::builder()
        .keys(doc! {
            "deviceUuid": 1,
            "featureUuid": 1,
            "featureName": 1,
        })
        .options(IndexOptions::builder().name("idx_device_feature_name".to_string()).unique(true).build())
        .build();
    match collection.create_index(index.clone()).await {
        Ok(_) => {
            info!(target: "app", "MongoDB indexes ensured");
        }
        // MongoDB error code 86 = IndexKeySpecsConflict (same name, different keys/options)
        Err(ref e) if matches!(e.kind.as_ref(), ErrorKind::Command(cmd) if cmd.code == 86) => {
            warn!(target: "app", "Index conflict detected for idx_device_feature_name, dropping and recreating");
            collection.drop_index("idx_device_feature_name").await.map_err(|e| {
                error!(target: "app", "MongoDB - cannot drop conflicting index: {}", e);
                e
            })?;
            collection.create_index(index).await?;
            info!(target: "app", "MongoDB indexes recreated successfully");
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

/// Retries `f` up to `max_retries` additional times after the first attempt
/// (total attempts = max_retries + 1), with a linearly increasing delay capped
/// at 30 seconds between attempts.
async fn retry_connect_mongodb<T, E, Fut, F>(mut f: F, max_retries: u32) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut count = 0u32;
    loop {
        let result = f().await;
        if result.is_ok() {
            info!(target: "app", "MongoDB connected!");
            return result;
        }
        if count >= max_retries {
            error!(target: "app", "Cannot connect to MongoDB, max tries reached");
            return result;
        }
        count += 1;
        let delay = Duration::from_secs(u64::from(count).min(30));
        warn!(target: "app", "MongoDB ping failed (count={}), retrying in {}s...", count, delay.as_secs());
        sleep(delay).await;
    }
}
