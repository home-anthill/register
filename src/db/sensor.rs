use tracing::{debug, error};

use mongodb::Database;
use mongodb::bson::{Bson, Document, doc};
use mongodb::error::{ErrorKind, WriteFailure};

use crate::errors::db_error::DbError;
use crate::models::feature_name::FeatureName;
use crate::models::inputs::RegisterInput;

use super::COLLECTION_NAME;

pub async fn insert_sensor(db: &Database, input: RegisterInput, feature_name: FeatureName) -> Result<String, DbError> {
    debug!(target: "app", "insert_sensor - Called with feature_name = {}, device_uuid = {}, feature_uuid = {}", feature_name, input.device_uuid, input.feature_uuid);

    let collection = db.collection::<Document>(COLLECTION_NAME);

    let serialized_input: Bson = input.into_sensor_bson(feature_name).map_err(|e| DbError::other(e.to_string()))?;

    debug!(target: "app", "insert_sensor - Adding sensor into db");

    let Bson::Document(document) = serialized_input else {
        return Err(DbError::other("Failed to convert sensor to BSON document"));
    };
    let insert_one_result = collection.insert_one(document).await.map_err(|err| {
        if matches!(err.kind.as_ref(), ErrorKind::Write(WriteFailure::WriteError(we)) if we.code == 11000) {
            return DbError::AlreadyExists;
        }
        error!(target: "app", "insert_sensor - MongoDB error: {}", err);
        DbError::other("Database operation failed")
    })?;
    let object_id = insert_one_result
        .inserted_id
        .as_object_id()
        .ok_or_else(|| DbError::other("Inserted ID is not a valid ObjectId"))?;
    Ok(object_id.to_hex())
}

pub async fn find_sensor_value_by_uuid(
    db: &Database,
    device_uuid: &str,
    feature_uuid: &str,
    feature_name: FeatureName,
) -> Result<Document, DbError> {
    debug!(target: "app", "find_sensor_value_by_uuid - Called with feature_name = {}, device_uuid = {}, feature_uuid = {}", feature_name, device_uuid, feature_uuid);
    let collection = db.collection::<Document>(COLLECTION_NAME);
    let filter = doc! {
        "deviceUuid": device_uuid,
        "featureUuid": feature_uuid,
        "featureName": feature_name.as_str(),
    };
    // limit the output to {"value", "createdAt" and "modifiedAt"}
    let projection = doc! {"_id": 0, "value": 1, "createdAt": 1, "modifiedAt": 1};

    debug!(target: "app", "find_sensor_value_by_uuid - Querying sensor value from db");

    collection
        .find_one(filter)
        .projection(projection)
        .await
        .map_err(|err| {
            error!(target: "app", "find_sensor_value_by_uuid - MongoDB error: {}", err);
            DbError::other("Database operation failed")
        })?
        .ok_or_else(|| DbError::other("Cannot find sensor"))
}
