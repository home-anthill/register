use std::env;
use std::str::FromStr;

use mongodb::bson::{Bson, Document, doc};
use mongodb::options::ClientOptions;
use mongodb::{Client, Database};
use tracing::info;

use register::db::COLLECTION_NAME;
use register::models::feature_name::FeatureName;
use register::models::inputs::RegisterInput;

pub async fn connect() -> mongodb::error::Result<Database> {
    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI is not found.");
    let mongo_db_name = String::from("sensors_test");

    let mut client_options = ClientOptions::parse(mongo_uri).await?;
    client_options.app_name = Some("register-test".to_string());
    let client = Client::with_options(client_options)?;
    let database = client.database(&mongo_db_name);

    info!("MongoDB testing connected!");

    Ok(database)
}

pub async fn drop_all_collections(db: &Database) {
    db.collection::<Document>(COLLECTION_NAME).drop().await.expect("drop 'sensors' collection");
}

pub async fn find_sensor_by_uuid(
    db: &Database,
    device_uuid: &str,
    feature_uuid: &str,
    feature_name: &str,
) -> mongodb::error::Result<Option<Document>> {
    let collection = db.collection::<Document>(COLLECTION_NAME);
    let filter = doc! {
        "deviceUuid": device_uuid,
        "featureUuid": feature_uuid,
        "featureName": feature_name,
    };
    collection.find_one(filter).await
}

pub async fn insert_sensor(db: &Database, input: RegisterInput, feature_name: &str) -> mongodb::error::Result<String> {
    let collection = db.collection::<Document>(COLLECTION_NAME);
    let feature_name = FeatureName::from_str(feature_name).expect("Valid feature type");
    let serialized_data: Bson = input.into_sensor_bson(feature_name).expect("Failed to serialize sensor");
    let document = serialized_data.as_document().expect("Expected BSON document");
    let insert_one_result = collection.insert_one(document.to_owned()).await?;
    Ok(insert_one_result.inserted_id.as_object_id().expect("Expected ObjectId").to_hex())
}

pub async fn update_sensor_float_value_by_uuid(
    db: &Database,
    device_uuid: &str,
    feature_uuid: &str,
    feature_name: &str,
    value: f64,
) -> mongodb::error::Result<Option<Document>> {
    let collection = db.collection::<Document>(COLLECTION_NAME);
    let filter = doc! {
        "deviceUuid": device_uuid,
        "featureUuid": feature_uuid,
        "featureName": feature_name
    };
    let update = doc! {"$set": {"value": value}};
    collection.find_one_and_update(filter, update).await
}

pub async fn update_sensor_int_value_by_uuid(
    db: &Database,
    device_uuid: &str,
    feature_uuid: &str,
    feature_name: &str,
    value: i64,
) -> mongodb::error::Result<Option<Document>> {
    let collection = db.collection::<Document>(COLLECTION_NAME);
    let filter = doc! {
        "deviceUuid": device_uuid,
        "featureUuid": feature_uuid,
        "featureName": feature_name
    };
    let update = doc! {"$set": {"value": value}};
    collection.find_one_and_update(filter, update).await
}
