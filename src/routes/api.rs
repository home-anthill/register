use std::str::FromStr;

use mongodb::Database;
use mongodb::bson::Document;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::{Json, json};
use tracing::{debug, error, info, warn};

use crate::db::sensor;
use crate::errors::api_error::ApiResponse;
use crate::errors::db_error::DbError;
use crate::models::feature_name::FeatureName;
use crate::models::inputs::{RegisterInput, validate_uuid_field};

/// keepalive
#[rocket::get("/keepalive")]
pub fn keep_alive() -> ApiResponse {
    ApiResponse { json: json!({ "alive": true }), status: Status::Ok }
}

/// register a new sensor
#[rocket::post("/sensors/register/<feature_name>", data = "<input>")]
pub async fn post_register(db: &State<Database>, input: Json<RegisterInput>, feature_name: &str) -> ApiResponse {
    let Ok(feature_name) = FeatureName::from_str(feature_name) else {
        return bad_request("Invalid sensor type");
    };
    if let Err(e) = input.validate() {
        return bad_request(&e.to_string());
    }
    info!(target: "app", "REST - POST - post_register feature_name = {}, device_uuid = {}, feature_uuid = {}", feature_name, input.device_uuid, input.feature_uuid);
    insert_register(db, input.into_inner(), feature_name).await
}

/// get sensor value by device and feature UUIDs and type
#[rocket::get("/sensors/<device_uuid>/features/<feature_uuid>/<feature_name>")]
pub async fn get_sensor_value(
    db: &State<Database>,
    device_uuid: &str,
    feature_uuid: &str,
    feature_name: &str,
) -> ApiResponse {
    let Ok(feature_name) = FeatureName::from_str(feature_name) else {
        return bad_request("Invalid sensor type");
    };
    if let Err(e) = validate_uuid_field(device_uuid, "device_uuid") {
        return bad_request(&e.to_string());
    }
    if let Err(e) = validate_uuid_field(feature_uuid, "feature_uuid") {
        return bad_request(&e.to_string());
    }
    info!(target: "app", "REST - GET - get_sensor_value feature_name = {}, device_uuid = {}, feature_uuid = {}", feature_name, device_uuid, feature_uuid);
    find_sensor_value(db, device_uuid, feature_uuid, feature_name).await
}

/// delete sensor value by device and feature UUIDs
#[rocket::delete("/sensors/<device_uuid>/features/<feature_uuid>")]
pub async fn delete_sensor(db: &State<Database>, device_uuid: &str, feature_uuid: &str) -> ApiResponse {
    if let Err(e) = validate_uuid_field(device_uuid, "device_uuid") {
        return bad_request(&e.to_string());
    }
    if let Err(e) = validate_uuid_field(feature_uuid, "feature_uuid") {
        return bad_request(&e.to_string());
    }
    info!(target: "app", "REST - DELETE - delete_sensor device_uuid = {}, feature_uuid = {}", device_uuid, feature_uuid);
    delete_sensor_value(db, device_uuid, feature_uuid).await
}

async fn insert_register(db: &State<Database>, input: RegisterInput, feature_name: FeatureName) -> ApiResponse {
    match sensor::insert_sensor(db, input, feature_name).await {
        Ok(register_doc_id) => {
            debug!(target: "app", "insert_register - document inserted with id = {}", register_doc_id);
            ApiResponse { json: json!({ "id": register_doc_id }), status: Status::Ok }
        }
        Err(DbError::AlreadyExists) => {
            warn!(target: "app", "insert_register - sensor already registered");
            error_response(Status::Conflict, "Sensor already registered")
        }
        Err(error) => {
            error!(target: "app", "insert_register - error = {}", error);
            internal_error()
        }
    }
}

async fn find_sensor_value(
    db: &State<Database>,
    device_uuid: &str,
    feature_uuid: &str,
    feature_name: FeatureName,
) -> ApiResponse {
    match sensor::find_sensor_value_by_uuid(db, device_uuid, feature_uuid, feature_name).await {
        Ok(sensor_doc) => {
            debug!(target: "app", "find_sensor_value - sensor document found");
            build_sensor_response(feature_name, &sensor_doc).unwrap_or_else(|e| e)
        }
        Err(error) => {
            error!(target: "app", "find_sensor_value - error {}", error);
            internal_error()
        }
    }
}

async fn delete_sensor_value(db: &State<Database>, device_uuid: &str, feature_uuid: &str) -> ApiResponse {
    match sensor::delete_sensor_by_uuid(db, device_uuid, feature_uuid).await {
        Ok(deleted_count) => {
            debug!(target: "app", "delete_sensor_value - deleted_count = {}", deleted_count);
            ApiResponse { json: json!({}), status: Status::Ok }
        }
        Err(error) => {
            error!(target: "app", "delete_sensor_value - error {}", error);
            internal_error()
        }
    }
}

fn build_sensor_response(feature_name: FeatureName, doc: &Document) -> Result<ApiResponse, ApiResponse> {
    let value = if feature_name.is_float() {
        let v = doc.get_f64("value").map_err(|err| {
            debug!(target: "app", "find_sensor_value - failed to get f64 value: {}", err);
            error!(target: "app", "find_sensor_value - unexpected value type in document");
            internal_error()
        })?;
        json!(v)
    } else {
        let v = doc.get_i64("value").map_err(|err| {
            debug!(target: "app", "find_sensor_value - failed to get i64 value: {}", err);
            error!(target: "app", "find_sensor_value - unexpected value type in document");
            internal_error()
        })?;
        json!(v)
    };
    let created_at = doc.get_datetime("createdAt").map(|dt| dt.timestamp_millis()).map_err(|err| {
        debug!(target: "app", "find_sensor_value - failed to get createdAt: {}", err);
        error!(target: "app", "find_sensor_value - unexpected createdAt type in document");
        internal_error()
    })?;
    let modified_at = doc.get_datetime("modifiedAt").map(|dt| dt.timestamp_millis()).map_err(|err| {
        debug!(target: "app", "find_sensor_value - failed to get modifiedAt: {}", err);
        error!(target: "app", "find_sensor_value - unexpected modifiedAt type in document");
        internal_error()
    })?;
    Ok(ApiResponse {
        json: json!({
            "value": value,
            "createdAt": created_at,
            "modifiedAt": modified_at,
        }),
        status: Status::Ok,
    })
}

fn error_response(status: Status, message: &str) -> ApiResponse {
    ApiResponse { json: json!({ "message": message, "code": status.code }), status }
}

fn internal_error() -> ApiResponse {
    error_response(Status::InternalServerError, "Internal server error")
}

fn bad_request(message: &str) -> ApiResponse {
    error_response(Status::BadRequest, message)
}

#[cfg(test)]
mod tests {
    use mongodb::bson::{DateTime, doc};
    use pretty_assertions::assert_eq;
    use rocket::http::Status;
    use rocket::serde::json::json;

    use super::build_sensor_response;
    use crate::models::feature_name::FeatureName;

    #[test]
    fn build_sensor_response_returns_float_value() {
        let now = DateTime::now();
        let doc = doc! {
            "value": 23.5,
            "createdAt": now,
            "modifiedAt": now,
        };

        let response = build_sensor_response(FeatureName::Temperature, &doc).unwrap();

        assert_eq!(response.status, Status::Ok);
        assert_eq!(
            response.json,
            json!({
                "value": 23.5,
                "createdAt": now.timestamp_millis(),
                "modifiedAt": now.timestamp_millis(),
            })
        );
    }

    #[test]
    fn build_sensor_response_returns_admitted_mode_float_values() {
        let now = DateTime::now();

        for value in [-1.0, 0.0, 1.0, 2.0] {
            let doc = doc! {
                "value": value,
                "createdAt": now,
                "modifiedAt": now,
            };

            let response = build_sensor_response(FeatureName::Mode, &doc).unwrap();

            assert_eq!(response.status, Status::Ok);
            assert_eq!(response.json["value"], json!(value));
        }
    }

    #[test]
    fn build_sensor_response_returns_internal_error_for_wrong_float_type() {
        let now = DateTime::now();
        let doc = doc! {
            "value": 1_i64,
            "createdAt": now,
            "modifiedAt": now,
        };

        let response = build_sensor_response(FeatureName::Temperature, &doc).expect_err("wrong value type must fail");

        assert_eq!(response.status, Status::InternalServerError);
        assert_eq!(response.json, json!({ "message": "Internal server error", "code": 500 }));
    }

    #[test]
    fn build_sensor_response_returns_internal_error_for_wrong_int_type() {
        let now = DateTime::now();
        let doc = doc! {
            "value": 1.5,
            "createdAt": now,
            "modifiedAt": now,
        };

        let response = build_sensor_response(FeatureName::Motion, &doc).expect_err("wrong value type must fail");

        assert_eq!(response.status, Status::InternalServerError);
        assert_eq!(response.json, json!({ "message": "Internal server error", "code": 500 }));
    }

    #[test]
    fn build_sensor_response_returns_internal_error_for_missing_modified_at() {
        let now = DateTime::now();
        let doc = doc! {
            "value": 1_i64,
            "createdAt": now,
        };

        let response = build_sensor_response(FeatureName::Motion, &doc).expect_err("missing date must fail");

        assert_eq!(response.status, Status::InternalServerError);
        assert_eq!(response.json, json!({ "message": "Internal server error", "code": 500 }));
    }
}
