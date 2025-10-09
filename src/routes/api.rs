use mongodb::Database;
use mongodb::bson::doc;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::{Json, json};
use tracing::{debug, error, info};

use crate::db::sensor;
use crate::errors::api_error::{ApiError, ApiResponse};
use crate::models::inputs::RegisterInput;

pub static VALID_SENSOR_TYPES: &[&str] = &[
    "temperature",
    "humidity",
    "light",
    "motion",
    "airquality",
    "airpressure",
    "poweroutage",
];

/// keepalive
#[get("/keepalive")]
pub async fn keep_alive() -> ApiResponse {
    ApiResponse {
        json: json!({ "alive": true }),
        code: Status::Ok.code,
    }
}

/// register a new sensor
#[post("/sensors/register/<sensor_type>", data = "<input>")]
pub async fn post_register(db: &State<Database>, input: Json<RegisterInput>, sensor_type: &str) -> ApiResponse {
    if VALID_SENSOR_TYPES.contains(&sensor_type) {
        info!(target: "app", "REST - POST - post_register sensor_type = {}", sensor_type);
        insert_register(db, input, sensor_type).await
    } else {
        ApiResponse {
            json: serde_json::to_value(ApiError {
                message: "Invalid sensor type".to_string(),
                code: Status::BadRequest.code,
            })
            .unwrap(),
            code: Status::BadRequest.code,
        }
    }
}

/// get sensor value by device and feature UUIDs and type
#[get("/sensors/<device_uuid>/features/<feature_uuid>/<sensor_type>")]
pub async fn get_sensor_value(
    db: &State<Database>,
    device_uuid: &str,
    feature_uuid: &str,
    sensor_type: &str,
) -> ApiResponse {
    info!(target: "app", "REST - GET - get_sensor_value sensor_type = {}, device_uuid = {}, feature_uuid = {}", sensor_type, device_uuid, feature_uuid);
    find_sensor_value(db, device_uuid, feature_uuid, sensor_type).await
}

async fn insert_register(db: &State<Database>, input: Json<RegisterInput>, sensor_type: &str) -> ApiResponse {
    debug!(target: "app", "insert_register - called with sensor_type = {}", sensor_type);
    match sensor::insert_sensor(db, input, sensor_type).await {
        Ok(register_doc_id) => {
            debug!(target: "app", "insert_register - document inserted with id = {}", register_doc_id);
            ApiResponse {
                json: json!({ "id": register_doc_id }),
                code: Status::Ok.code,
            }
        }
        Err(error) => {
            error!(target: "app", "insert_register - error = {:?}", error);
            ApiResponse {
                json: serde_json::to_value(ApiError {
                    message: "Invalid input".to_string(),
                    code: Status::BadRequest.code,
                })
                .unwrap(),
                code: Status::BadRequest.code,
            }
        }
    }
}

async fn find_sensor_value(
    db: &State<Database>,
    device_uuid: &str,
    feature_uuid: &str,
    sensor_type: &str,
) -> ApiResponse {
    match sensor::find_sensor_value_by_uuid(db, device_uuid, feature_uuid, sensor_type).await {
        Ok(sensor_doc) => {
            info!(target: "app", "find_sensor_value - result sensor_doc = {}", sensor_doc);
            let value: f64 = match sensor_type {
                "temperature" | "humidity" | "light" | "airpressure" => sensor_doc.get_f64("value").unwrap(),
                "motion" | "airquality" | "poweroutage" => sensor_doc.get_i64("value").unwrap() as f64,
                _ => {
                    return ApiResponse {
                        json: serde_json::to_value(ApiError {
                            message: "Unknown sensor type".to_string(),
                            code: Status::InternalServerError.code,
                        })
                        .unwrap(),
                        code: Status::InternalServerError.code,
                    };
                }
            };
            let created_at = sensor_doc.get_datetime("createdAt").unwrap().timestamp_millis();
            let modified_at = sensor_doc.get_datetime("modifiedAt").unwrap().timestamp_millis();
            ApiResponse {
                json: json!({
                    // in json response, 'value' is always a f64, even if in db it's a i64
                    "value": value,
                    "createdAt": created_at,
                    "modifiedAt": modified_at,
                }),
                code: Status::Ok.code,
            }
        }
        Err(error) => {
            error!(target: "app", "find_sensor_value - error {:?}", error);
            ApiResponse {
                json: serde_json::to_value(ApiError {
                    message: "Internal server error".to_string(),
                    code: Status::InternalServerError.code,
                })
                .unwrap(),
                code: Status::InternalServerError.code,
            }
        }
    }
}
