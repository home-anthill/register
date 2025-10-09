use std::env;

use mongodb::bson::{Bson, Document, doc};
use mongodb::options::ClientOptions;
use mongodb::{Client, Database};
use rocket::serde::json::Json as RocketJson;

use register::models::inputs::RegisterInput;
use register::models::sensor::{FloatSensor, IntSensor, new_from_register_input};

pub async fn connect() -> mongodb::error::Result<Database> {
    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI is not found.");
    let mongo_db_name = String::from("sensors_test");

    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("register-test".to_string());
    let client = Client::with_options(client_options)?;
    let database = client.database(mongo_db_name.as_str());

    info!("MongoDB testing connected!");

    Ok(database)
}

pub async fn drop_all_collections(db: &Database) -> () {
    db.collection::<Document>("sensors")
        .drop()
        .await
        .expect("drop 'sensors' collection");
}

pub async fn find_sensor_by_uuid(
    db: &Database,
    device_uuid: &String,
    feature_uuid: &String,
    sensor_type: &str,
) -> mongodb::error::Result<Option<Document>> {
    let collection = db.collection::<Document>("sensors");
    let filter = doc! {
        "deviceUuid": device_uuid,
        "featureUuid": feature_uuid,
        "featureName": sensor_type,
    };
    collection.find_one(filter).await
}

pub async fn insert_sensor(
    db: &Database,
    input: RocketJson<RegisterInput>,
    sensor_type: &str,
) -> mongodb::error::Result<String> {
    let collection = db.collection::<Document>("sensors");
    let serialized_data: Bson = match sensor_type {
        "temperature" | "humidity" | "light" => new_from_register_input::<FloatSensor>(input, sensor_type).unwrap(),
        "motion" | "airquality" | "airpressure" => new_from_register_input::<IntSensor>(input, sensor_type).unwrap(),
        _ => {
            panic!("Unknown type")
        }
    };
    let document = serialized_data.as_document().unwrap();
    let insert_one_result = collection.insert_one(document.to_owned()).await?;
    Ok(insert_one_result.inserted_id.as_object_id().unwrap().to_hex())
}

pub async fn update_sensor_float_value_by_uuid(
    db: &Database,
    device_uuid: &String,
    feature_uuid: &String,
    sensor_type: &str,
    value: f64,
) -> mongodb::error::Result<Option<Document>> {
    let collection = db.collection::<Document>("sensors");
    let filter = doc! {
        "deviceUuid": device_uuid,
        "featureUuid": feature_uuid,
        "featureName": sensor_type
    };
    let update = doc! {"$set": {"value": value}};
    collection.find_one_and_update(filter.clone(), update).await
}

pub async fn update_sensor_int_value_by_uuid(
    db: &Database,
    device_uuid: &String,
    feature_uuid: &String,
    sensor_type: &str,
    value: i64,
) -> mongodb::error::Result<Option<Document>> {
    let collection = db.collection::<Document>("sensors");
    let filter = doc! {
        "deviceUuid": device_uuid,
        "featureUuid": feature_uuid,
        "featureName": sensor_type
    };
    let update = doc! {"$set": {"value": value}};
    collection.find_one_and_update(filter.clone(), update).await
}
