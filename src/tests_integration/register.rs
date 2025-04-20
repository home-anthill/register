use log::info;
use std::collections::HashMap;
use uuid::Uuid;

use super::rocket;
use mongodb::Database;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalRequest, LocalResponse};
use rocket::serde::json::Json;
use serde_json::{Value, json};

use crate::tests_integration::db_utils::{
    connect, drop_all_collections, find_sensor_by_uuid, insert_sensor, update_sensor_float_value_by_uuid,
    update_sensor_int_value_by_uuid,
};
use crate::tests_integration::test_utils::{build_register_input, create_register_input, get_random_mac};
use register::models::inputs::RegisterInput;

#[rocket::async_test]
async fn register_sensor() {
    // init
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    // run tests for every sensor_type
    let sensor_types = vec![
        "temperature",
        "humidity",
        "light",
        "airpressure",
        "motion",
        "airquality",
    ];
    for sensor_type in sensor_types.into_iter() {
        // inputs
        let sensor_uuid: String = Uuid::new_v4().to_string();
        let mac: String = get_random_mac();
        let profile_owner_id = String::from("63963ce7c7fd6d463c6c77a3");
        let register_body = build_register_input(&sensor_uuid, &mac, &profile_owner_id);

        // test api
        let req: LocalRequest = client
            .post("/sensors/register/".to_owned() + sensor_type)
            .header(ContentType::JSON)
            .body(register_body);
        let res: LocalResponse = req.dispatch().await;

        let document = find_sensor_by_uuid(&db, &sensor_uuid, sensor_type)
            .await
            .unwrap()
            .unwrap();
        let inserted_id = document.get_object_id("_id").unwrap().to_hex();

        // check results
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(res.into_json::<Value>().await.unwrap(), json!({ "id": inserted_id }));
    }

    // cleanup
    drop_all_collections(&db).await;
}

#[rocket::async_test]
async fn register_sensor_error() {
    // init
    let client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    // run tests for every sensor_type
    let sensor_types = vec![
        "temperature",
        "humidity",
        "light",
        "airpressure",
        "motion",
        "airquality",
    ];
    for sensor_type in sensor_types.into_iter() {
        // inputs
        let sensor_uuid: String = Uuid::new_v4().to_string();
        let mac: String = get_random_mac();
        let wrong_profile_id = String::from("dasd7dasjdhdsygsyuad");
        // try to add a sensor with POST body using a 'profileOwnerId'
        // with bad format (it must be a mongodb ObjectId)
        let register_body = build_register_input(&sensor_uuid, &mac, &wrong_profile_id);

        // test api
        let req: LocalRequest = client
            .post("/sensors/register/".to_owned() + sensor_type)
            .header(ContentType::JSON)
            .body(register_body);
        let res: LocalResponse = req.dispatch().await;

        // check results
        assert_eq!(res.status(), Status::BadRequest);
    }

    // cleanup
    drop_all_collections(&db).await;
}

#[rocket::async_test]
async fn get_float_sensor_value() {
    // init
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    // run tests for every sensor_type
    let sensors_inputs: HashMap<String, f64> = HashMap::from([
        (String::from("temperature"), 28.12),
        (String::from("humidity"), 67_f64),
        (String::from("light"), 12_f64),
        (String::from("airpressure"), 10.99),
    ]);

    for (sensor_type, sensor_val) in &sensors_inputs {
        info!(target: "test", "get_sensor_value - TEST with type = {} and value = {}", &sensor_type, sensor_val);
        // inputs
        let sensor_uuid: String = Uuid::new_v4().to_string();
        let mac: String = get_random_mac();
        let profile_owner_id = "63963ce7c7fd6d463c6c77a3";
        let register_body: RegisterInput = create_register_input(&sensor_uuid, &mac, profile_owner_id);

        // fill db with a sensor with default zero value
        let _ = insert_sensor(&db, Json(register_body), sensor_type).await;
        update_sensor_float_value_by_uuid(&db, &sensor_uuid, sensor_type, *sensor_val)
            .await
            .unwrap()
            .unwrap();
        // read again the sensor document, previously updated
        let document = find_sensor_by_uuid(&db, &sensor_uuid, sensor_type)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(document.get("value").unwrap().as_f64().unwrap(), *sensor_val);

        // read dates from db
        let created_at = document.get_datetime("createdAt").unwrap().timestamp_millis();
        let modified_at = document.get_datetime("modifiedAt").unwrap().timestamp_millis();

        // test api
        let req: LocalRequest = client.get(format!("/sensors/{}/{}", sensor_uuid, sensor_type));
        let res: LocalResponse = req.dispatch().await;

        // check results
        assert_eq!(res.status(), Status::Ok);
        let expected = json!({
            "value": *sensor_val,
            "createdAt": created_at,
            "modifiedAt": modified_at,
        });
        assert_eq!(res.into_json::<Value>().await.unwrap(), expected);
    }

    // cleanup
    drop_all_collections(&db).await;
}

#[rocket::async_test]
async fn get_int_sensor_value() {
    // init
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    // run tests for every sensor_type
    let sensors_inputs: HashMap<String, i64> =
        HashMap::from([(String::from("motion"), 1), (String::from("airquality"), 2)]);

    for (sensor_type, sensor_val) in &sensors_inputs {
        info!(target: "test", "get_sensor_value - TEST with type = {} and value = {}", &sensor_type, sensor_val);
        // inputs
        let sensor_uuid: String = Uuid::new_v4().to_string();
        let mac: String = get_random_mac();
        let profile_owner_id = "63963ce7c7fd6d463c6c77a3";
        let register_body: RegisterInput = create_register_input(&sensor_uuid, &mac, profile_owner_id);

        // fill db with a sensor with default zero value
        let _ = insert_sensor(&db, Json(register_body), sensor_type).await;
        update_sensor_int_value_by_uuid(&db, &sensor_uuid, sensor_type, *sensor_val)
            .await
            .unwrap()
            .unwrap();
        // read again the sensor document, previously updated
        let document = find_sensor_by_uuid(&db, &sensor_uuid, sensor_type)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(document.get_i64("value").unwrap(), *sensor_val);

        // read dates from db
        let created_at = document.get_datetime("createdAt").unwrap().timestamp_millis();
        let modified_at = document.get_datetime("modifiedAt").unwrap().timestamp_millis();

        // test api
        let req: LocalRequest = client.get(format!("/sensors/{}/{}", sensor_uuid, sensor_type));
        let res: LocalResponse = req.dispatch().await;

        // check results
        assert_eq!(res.status(), Status::Ok);
        let expected = json!({
            "value": *sensor_val as f64,
            "createdAt": created_at,
            "modifiedAt": modified_at,
        });
        assert_eq!(res.into_json::<Value>().await.unwrap(), expected);
    }

    // cleanup
    drop_all_collections(&db).await;
}
