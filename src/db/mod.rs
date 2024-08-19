use crate::config::Env;
use std::env;
use log::{error, info, warn};
use mongodb::bson::doc;
use mongodb::options::{ClientOptions, ServerApi, ServerApiVersion};
use mongodb::{Client, Database};
use rocket::fairing::AdHoc;

pub mod sensor;

pub fn init(env_config: Env) -> AdHoc {
    AdHoc::on_ignite("Connecting to MongoDB", |rocket| async {
        match connect(env_config).await {
            Ok(database) => rocket.manage(database),
            Err(error) => {
                error!(target: "app", "MongoDB - cannot connect {:?}", error);
                panic!("Cannot connect to MongoDB:: {:?}", error)
            }
        }
    })
}

async fn connect(env_config: Env) -> mongodb::error::Result<Database> {
    let mongo_uri = env_config.mongo_uri.clone();

    let mongo_db_name = if env::var("ENV") == Ok(String::from("testing")) {
        warn!("TESTING ENVIRONMENT - forcing mongo_db_name = 'sensors_test'");
        String::from("sensors_test")
    } else {
        env_config.mongo_db_name.clone()
    };

    let mut client_options = ClientOptions::parse(mongo_uri).await?;

    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Set app_name
    client_options.app_name = Some("register".to_string());

    // Create a new client and connect to the server
    let client = Client::with_options(client_options)?;
    let database = client.database(mongo_db_name.as_str());

    info!(target: "app", "Pinging MongoDB server...");
    database.run_command(doc! { "ping": 1 }).await?;

    info!(target: "app", "MongoDB connected!");

    Ok(database)
}
