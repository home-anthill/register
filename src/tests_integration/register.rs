use std::collections::HashMap;

use super::rocket;
use mongodb::Database;
use mongodb::bson::{Document, doc};
use pretty_assertions::assert_eq;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::{Client, LocalRequest, LocalResponse};
use serde_json::{Value, json};
use tracing::info;
use uuid::Uuid;

use register::db::COLLECTION_NAME;
use register::models::feature_name::FeatureName;
use register::models::inputs::RegisterInput;

use crate::tests_integration::db_utils::{
    connect, drop_all_collections, find_sensor_by_uuid, insert_sensor, update_sensor_float_value_by_uuid,
    update_sensor_int_value_by_uuid,
};
use crate::tests_integration::test_utils::{
    build_register_input, build_register_input_with_token, create_register_input, get_random_mac,
};

#[rocket::async_test]
#[test_log::test]
async fn register_sensor() {
    // init
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    for feature_type in FeatureName::ALL {
        // inputs
        let profile_owner_id = String::from("63963ce7c7fd6d463c6c77a3");
        let device_uuid: String = Uuid::new_v4().to_string();
        let mac: String = get_random_mac();
        let feature_uuid: String = Uuid::new_v4().to_string();
        let register_body = build_register_input(&profile_owner_id, &device_uuid, &mac, &feature_uuid);

        // test api
        let req: LocalRequest =
            client.post(format!("/sensors/register/{}", feature_type)).header(ContentType::JSON).body(register_body);
        let res: LocalResponse = req.dispatch().await;

        let document =
            find_sensor_by_uuid(&db, &device_uuid, &feature_uuid, feature_type.as_str()).await.unwrap().unwrap();
        let inserted_id = document.get_object_id("_id").unwrap().to_hex();

        // check results
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(res.into_json::<Value>().await.unwrap(), json!({ "id": inserted_id }));
    }

    // cleanup
    drop_all_collections(&db).await;
}

#[rocket::async_test]
#[test_log::test]
async fn register_thermostat_with_mode() {
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    let profile_owner_id = "63963ce7c7fd6d463c6c77a3";
    let device_uuid = Uuid::new_v4().to_string();
    let mac = get_random_mac();

    for feature_name in ["temperature", "mode"] {
        let feature_uuid = Uuid::new_v4().to_string();
        let mut input = create_register_input(profile_owner_id, &device_uuid, &mac, &feature_uuid);
        input.model = "thermostat".to_string();

        let response = client
            .post(format!("/sensors/register/{feature_name}"))
            .header(ContentType::JSON)
            .body(serde_json::to_string(&input).unwrap())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert!(find_sensor_by_uuid(&db, &device_uuid, &feature_uuid, feature_name).await.unwrap().is_some());
    }

    let registered_features =
        db.collection::<Document>(COLLECTION_NAME).count_documents(doc! { "deviceUuid": &device_uuid }).await.unwrap();
    assert_eq!(registered_features, 2);

    drop_all_collections(&db).await;
}

#[rocket::async_test]
#[test_log::test]
async fn register_thermostat_without_mode() {
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    let profile_owner_id = "63963ce7c7fd6d463c6c77a3";
    let device_uuid = Uuid::new_v4().to_string();
    let mac = get_random_mac();
    let feature_uuid = Uuid::new_v4().to_string();
    let mut input = create_register_input(profile_owner_id, &device_uuid, &mac, &feature_uuid);
    input.model = "thermostat".to_string();

    let response = client
        .post("/sensors/register/temperature")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&input).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), Status::Ok);
    assert!(find_sensor_by_uuid(&db, &device_uuid, &feature_uuid, "temperature").await.unwrap().is_some());

    let registered_modes = db
        .collection::<Document>(COLLECTION_NAME)
        .count_documents(doc! { "deviceUuid": &device_uuid, "featureName": "mode" })
        .await
        .unwrap();
    assert_eq!(registered_modes, 0);

    drop_all_collections(&db).await;
}

#[rocket::async_test]
#[test_log::test]
async fn register_sensor_wrong_profile_error() {
    // init
    let client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    // run tests for every feature_type
    for feature_type in FeatureName::ALL {
        // inputs
        let wrong_profile_id = String::from("dasd7dasjdhdsygsyuad");
        let device_uuid: String = Uuid::new_v4().to_string();
        let mac: String = get_random_mac();
        let feature_uuid: String = Uuid::new_v4().to_string();
        // try to add a sensor with POST body using a 'profileOwnerId'
        // with bad format (it must be a mongodb ObjectId)
        let register_body = build_register_input(&wrong_profile_id, &device_uuid, &mac, &feature_uuid);
        // test api
        let req: LocalRequest =
            client.post(format!("/sensors/register/{}", feature_type)).header(ContentType::JSON).body(register_body);
        let res: LocalResponse = req.dispatch().await;

        // check results
        assert_eq!(res.status(), Status::BadRequest);
    }

    // cleanup
    drop_all_collections(&db).await;
}

#[rocket::async_test]
#[test_log::test]
async fn register_sensor_invalid_api_token_error() {
    // init
    let client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    let invalid_tokens = &[
        "not-a-uuid",
        "12345678-1234-1234-1234-123456789012", // UUID v1, not v4
        "",
        "473a4861-632b-4915-b01e-cf1d418966c6-extra",
        "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx",
    ];

    for feature_type in FeatureName::ALL {
        for invalid_token in invalid_tokens {
            let profile_owner_id = String::from("63963ce7c7fd6d463c6c77a3");
            let device_uuid: String = Uuid::new_v4().to_string();
            let mac: String = get_random_mac();
            let feature_uuid: String = Uuid::new_v4().to_string();
            let register_body =
                build_register_input_with_token(&profile_owner_id, &device_uuid, &mac, &feature_uuid, invalid_token);

            let req: LocalRequest = client
                .post(format!("/sensors/register/{}", feature_type))
                .header(ContentType::JSON)
                .body(register_body);
            let res: LocalResponse = req.dispatch().await;

            assert_eq!(res.status(), Status::BadRequest);
        }
    }

    // cleanup
    drop_all_collections(&db).await;
}

#[rocket::async_test]
#[test_log::test]
async fn register_sensor_valid_api_token_uuid_v4() {
    // init
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    for feature_type in FeatureName::ALL {
        let profile_owner_id = String::from("63963ce7c7fd6d463c6c77a3");
        let device_uuid: String = Uuid::new_v4().to_string();
        let mac: String = get_random_mac();
        let feature_uuid: String = Uuid::new_v4().to_string();
        let valid_token = Uuid::new_v4().to_string();
        let register_body =
            build_register_input_with_token(&profile_owner_id, &device_uuid, &mac, &feature_uuid, &valid_token);

        let req: LocalRequest =
            client.post(format!("/sensors/register/{}", feature_type)).header(ContentType::JSON).body(register_body);
        let res: LocalResponse = req.dispatch().await;

        assert_eq!(res.status(), Status::Ok);
    }

    // cleanup
    drop_all_collections(&db).await;
}

#[rocket::async_test]
#[test_log::test]
async fn register_sensor_wrong_type_error() {
    // init
    let client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    // inputs
    let feature_type = "unknown".to_string();
    let profile_owner_id = String::from("63963ce7c7fd6d463c6c77a3");
    let device_uuid: String = Uuid::new_v4().to_string();
    let mac: String = get_random_mac();
    let feature_uuid: String = Uuid::new_v4().to_string();
    // try to add a sensor with a bad type
    let register_body = build_register_input(&profile_owner_id, &device_uuid, &mac, &feature_uuid);
    // test api
    let req: LocalRequest =
        client.post(format!("/sensors/register/{}", feature_type)).header(ContentType::JSON).body(register_body);
    let res: LocalResponse = req.dispatch().await;

    // check results
    assert_eq!(res.status(), Status::BadRequest);

    // cleanup
    drop_all_collections(&db).await;
}

#[rocket::async_test]
#[test_log::test]
async fn get_float_sensor_value() {
    // init
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    // run tests for every feature_type
    let sensors_inputs: HashMap<String, f64> = HashMap::from([
        (String::from("temperature"), 28.12),
        (String::from("humidity"), 67_f64),
        (String::from("light"), 12_f64),
        (String::from("airpressure"), 10.99),
    ]);

    for (feature_type, sensor_val) in &sensors_inputs {
        info!(target: "test", "get_sensor_value - TEST with type = {} and value = {}", &feature_type, sensor_val);
        // inputs
        let profile_owner_id = String::from("63963ce7c7fd6d463c6c77a3");
        let device_uuid: String = Uuid::new_v4().to_string();
        let mac: String = get_random_mac();
        let feature_uuid: String = Uuid::new_v4().to_string();
        let register_body: RegisterInput = create_register_input(&profile_owner_id, &device_uuid, &mac, &feature_uuid);

        // fill db with a sensor with default zero value
        let _ = insert_sensor(&db, register_body, feature_type).await;
        update_sensor_float_value_by_uuid(&db, &device_uuid, &feature_uuid, feature_type, *sensor_val)
            .await
            .unwrap()
            .unwrap();
        // read again the sensor document, previously updated
        let document = find_sensor_by_uuid(&db, &device_uuid, &feature_uuid, feature_type).await.unwrap().unwrap();
        assert_eq!(document.get("value").unwrap().as_f64().unwrap(), *sensor_val);

        // read dates from db
        let created_at = document.get_datetime("createdAt").unwrap().timestamp_millis();
        let modified_at = document.get_datetime("modifiedAt").unwrap().timestamp_millis();

        // test api
        let req: LocalRequest =
            client.get(format!("/sensors/{}/features/{}/{}", device_uuid, feature_uuid, feature_type));
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
#[test_log::test]
async fn get_int_sensor_value() {
    // init
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    // run tests for every feature_type
    let sensors_inputs: HashMap<String, i64> =
        HashMap::from([(String::from("motion"), 1), (String::from("airquality"), 2), (String::from("mode"), 2)]);

    for (feature_type, sensor_val) in &sensors_inputs {
        info!(target: "test", "get_sensor_value - TEST with type = {} and value = {}", &feature_type, sensor_val);
        // inputs
        let profile_owner_id = String::from("63963ce7c7fd6d463c6c77a3");
        let device_uuid: String = Uuid::new_v4().to_string();
        let mac: String = get_random_mac();
        let feature_uuid: String = Uuid::new_v4().to_string();
        let register_body: RegisterInput = create_register_input(&profile_owner_id, &device_uuid, &mac, &feature_uuid);
        // fill db with a sensor with default zero value
        let _ = insert_sensor(&db, register_body, feature_type).await;
        update_sensor_int_value_by_uuid(&db, &device_uuid, &feature_uuid, feature_type, *sensor_val)
            .await
            .unwrap()
            .unwrap();
        // read again the sensor document, previously updated
        let document = find_sensor_by_uuid(&db, &device_uuid, &feature_uuid, feature_type).await.unwrap().unwrap();
        assert_eq!(document.get_i64("value").unwrap(), *sensor_val);

        // read dates from db
        let created_at = document.get_datetime("createdAt").unwrap().timestamp_millis();
        let modified_at = document.get_datetime("modifiedAt").unwrap().timestamp_millis();

        // test api
        let req: LocalRequest =
            client.get(format!("/sensors/{}/features/{}/{}", device_uuid, feature_uuid, feature_type));
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
#[test_log::test]
async fn delete_sensor() {
    // init
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    let feature_type = "online";
    let profile_owner_id = String::from("63963ce7c7fd6d463c6c77a3");
    let device_uuid: String = Uuid::new_v4().to_string();
    let mac: String = get_random_mac();
    let feature_uuid: String = Uuid::new_v4().to_string();
    let register_body: RegisterInput = create_register_input(&profile_owner_id, &device_uuid, &mac, &feature_uuid);

    let _ = insert_sensor(&db, register_body, feature_type).await;
    let document = find_sensor_by_uuid(&db, &device_uuid, &feature_uuid, feature_type).await.unwrap();
    assert!(document.is_some());

    let req: LocalRequest = client.delete(format!("/sensors/{}/features/{}", device_uuid, feature_uuid));
    let res: LocalResponse = req.dispatch().await;

    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.into_json::<Value>().await.unwrap(), json!({}));
    let document = find_sensor_by_uuid(&db, &device_uuid, &feature_uuid, feature_type).await.unwrap();
    assert!(document.is_none());

    // cleanup
    drop_all_collections(&db).await;
}

#[rocket::async_test]
#[test_log::test]
async fn delete_sensor_returns_ok_when_missing() {
    // init
    let client: Client = Client::tracked(rocket()).await.unwrap();
    let db: Database = connect().await.unwrap();
    drop_all_collections(&db).await;

    let device_uuid: String = Uuid::new_v4().to_string();
    let feature_uuid: String = Uuid::new_v4().to_string();

    let req: LocalRequest = client.delete(format!("/sensors/{}/features/{}", device_uuid, feature_uuid));
    let res: LocalResponse = req.dispatch().await;

    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.into_json::<Value>().await.unwrap(), json!({}));

    // cleanup
    drop_all_collections(&db).await;
}
