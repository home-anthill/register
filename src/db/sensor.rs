use tracing::{debug, error, info};

use mongodb::Database;
use mongodb::bson::{Bson, Document, doc};
use rocket::serde::json::Json;

use crate::errors::db_error::DbError;
use crate::models::inputs::RegisterInput;
use crate::models::sensor::{FloatSensor, IntSensor, new_from_register_input};

pub async fn insert_sensor(db: &Database, input: Json<RegisterInput>, sensor_type: &str) -> Result<String, DbError> {
    info!(target: "app", "insert_sensor - Called with sensor_type = {}", sensor_type);

    let collection = db.collection::<Document>("sensors");

    let serialized_input: Bson = match sensor_type {
        "temperature" | "humidity" | "light" | "airpressure" => {
            let result = new_from_register_input::<FloatSensor>(input, sensor_type);
            match result {
                Ok(res) => res,
                Err(err) => return Err(DbError::new(err.to_string())),
            }
        }
        "motion" | "airquality" | "online" => {
            let result = new_from_register_input::<IntSensor>(input, sensor_type);
            match result {
                Ok(res) => res,
                Err(err) => return Err(DbError::new(err.to_string())),
            }
        }
        _ => {
            error!(target: "app", "insert_sensor - Unknown sensor_type = {}", sensor_type);
            return Err(DbError::new(format!("Unknown sensor_type = {}", sensor_type)));
        }
    };

    debug!(target: "app", "insert_sensor - Adding sensor into db");

    let document = serialized_input.as_document().unwrap();
    let insert_one_result = collection.insert_one(document.to_owned()).await.unwrap();
    Ok(insert_one_result.inserted_id.as_object_id().unwrap().to_hex())
}

pub async fn find_sensor_value_by_uuid(
    db: &Database,
    device_uuid: &str,
    sensor_uuid: &str,
    sensor_type: &str,
) -> Result<Document, DbError> {
    info!(target: "app", "find_sensor_value_by_uuid - Called with sensor_type = {}, device_uuid = {}, sensor_uuid = {}", sensor_type, device_uuid, sensor_uuid);
    let collection = db.collection::<Document>("sensors");

    // find by uuid
    let filter = doc! {
        "deviceUuid": device_uuid,
        "featureUuid": sensor_uuid,
        "featureName": sensor_type,
    };
    // limit the output to {"value", "createdAt" and "modifiedAt"}
    let projection = doc! {"_id": 0, "value": 1, "createdAt": 1, "modifiedAt": 1};

    debug!(target: "app", "find_sensor_value_by_uuid - Getting sensor value with device_uuid = {} and sensor_uuid = {} from db", device_uuid, sensor_uuid);

    match collection.find_one(filter).projection(projection).await {
        Ok(doc_result) => match doc_result {
            Some(doc) => Ok(doc),
            None => Err(DbError::new(String::from("Cannot find sensor"))),
        },
        Err(err) => Err(DbError::new(err.to_string())),
    }
}
