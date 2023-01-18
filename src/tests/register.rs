use std::collections::HashMap;
use log::info;
use uuid::Uuid;

use super::rocket;
use mongodb::Database;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalRequest, LocalResponse};
use rocket::serde::json::Json;
use serde_json::{json, Value};

use crate::tests::db_utils::{connect, drop_all_collections, find_sensor_by_uuid, insert_sensor, update_sensor_float_value_by_uuid, update_sensor_int_value_by_uuid};
use crate::tests::test_utils::{build_register_input, create_register_input, get_random_mac};
use register::models::inputs::RegisterInput;

#[derive(Hash, Eq, PartialEq, Debug)]
struct SensorTypeTest {
    val_type: String,
    value: i64, // this should be a i64 or f64, but in this test I don't care
}

impl SensorTypeTest {
    fn new(val_type: &str, value: i64) -> SensorTypeTest {
        SensorTypeTest {
            val_type: String::from(val_type),
            value,
        }
    }
}

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
        "motion",
        "airquality",
        "airpressure",
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
        assert_eq!(
            res.into_json::<Value>().await.unwrap(),
            json!({ "id": inserted_id })
        );
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

    // inputs
    let sensor_uuid: String = Uuid::new_v4().to_string();
    let mac: String = get_random_mac();
    let wrong_profile_id = String::from("dasd7dasjdhdsygsyuad");
    // try to add a sensor with POST body using a 'profileOwnerId'
    // with bad format (it must be a mongodb ObjectId)
    let register_body = build_register_input(&sensor_uuid, &mac, &wrong_profile_id);

    // test api
    let req: LocalRequest = client
        .post("/sensors/register/temperature")
        .header(ContentType::JSON)
        .body(register_body);
    let res: LocalResponse = req.dispatch().await;

    // check results
    assert_eq!(res.status(), Status::BadRequest);

    // cleanup
    drop_all_collections(&db).await;
}

#[rocket::async_test]
async fn get_sensor_value() {
    // init
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    // run tests for every sensor_type
    // Use a HashMap to store the vikings' health points.
    let sensors_inputs: HashMap<String, SensorTypeTest> = HashMap::from([
        (String::from("temperature"), SensorTypeTest::new("f64", 28)),
        (String::from("humidity"), SensorTypeTest::new("f64", 67)),
        (String::from("light"), SensorTypeTest::new("f64", 12)),
        (String::from("motion"), SensorTypeTest::new("i64", 1)),
        (String::from("airquality"), SensorTypeTest::new("i64", 2)),
        (String::from("airpressure"), SensorTypeTest::new("f64", 100)),
    ]);

    for (sensor_type, sensor_val) in &sensors_inputs {
        info!(target: "test", "get_sensor_value - TEST with type = {}, val_type = {} and value = {}", &sensor_type, &sensor_val.val_type, &sensor_val.value);
        // inputs
        let sensor_uuid: String = Uuid::new_v4().to_string();
        let mac: String = get_random_mac();
        let profile_owner_id = "63963ce7c7fd6d463c6c77a3";
        let register_body: RegisterInput = create_register_input(&sensor_uuid, &mac, profile_owner_id);

        // fill db with a sensor with default zero value
        let _ = insert_sensor(&db, Json(register_body), sensor_type).await;
        if sensor_val.val_type == "f64" {
            update_sensor_float_value_by_uuid(
                &db,
                &sensor_uuid,
                sensor_type,
                sensor_val.value as f64,
            )
            .await
            .unwrap()
            .unwrap();
            let document = find_sensor_by_uuid(&db, &sensor_uuid, sensor_type)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                document.get("value").unwrap().as_f64().unwrap(),
                sensor_val.value as f64
            );
        } else if sensor_val.val_type == "i64" {
            update_sensor_int_value_by_uuid(
                &db,
                &sensor_uuid,
                sensor_type,
                sensor_val.value as i64,
            )
            .await
            .unwrap()
            .unwrap();
            let document = find_sensor_by_uuid(&db, &sensor_uuid, sensor_type)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                document.get("value").unwrap().as_i64().unwrap(),
                sensor_val.value
            );
        } else {
            panic!("Unknown val_type. It must be either 'f64' or 'i64'.");
        }

        // test api
        let req: LocalRequest = client.get(format!("/sensors/{}/{}", sensor_uuid, sensor_type));
        let res: LocalResponse = req.dispatch().await;

        // check results
        assert_eq!(res.status(), Status::Ok);
        let expected = if sensor_val.val_type == "f64" {
            json!({ "value": sensor_val.value as f64 })
        } else if sensor_val.val_type == "i64" {
            json!({ "value": sensor_val.value as i64 })
        } else {
            panic!("Unknown val_type. It must be either 'f64' or 'i64'.");
        };
        assert_eq!(res.into_json::<Value>().await.unwrap(), expected);
    }

    // cleanup
    drop_all_collections(&db).await;
}
