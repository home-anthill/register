use std::str::FromStr;

use mongodb::bson::oid::ObjectId;
use mongodb::bson::{Bson, DateTime, oid, to_bson};
use oid::Error;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::models::inputs::RegisterInput;

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntSensor {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    // profile info
    pub profileOwnerId: ObjectId,
    pub apiToken: String,
    // device info
    pub deviceUuid: String,
    pub mac: String,
    pub model: String,
    pub manufacturer: String,
    // feature info
    pub featureUuid: String,
    pub featureName: String,
    pub value: i64,
    // dates
    pub createdAt: DateTime,
    pub modifiedAt: DateTime,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FloatSensor {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    // profile info
    pub profileOwnerId: ObjectId,
    pub apiToken: String,
    // device info
    pub deviceUuid: String,
    pub mac: String,
    pub model: String,
    pub manufacturer: String,
    // feature info
    pub featureUuid: String,
    pub featureName: String,
    pub value: f64,
    // dates
    pub createdAt: DateTime,
    pub modifiedAt: DateTime,
}

pub trait Sensor {
    fn new(
        // profile info
        profile_owner_id: ObjectId,
        api_token: String,
        // device info
        device_uuid: String,
        mac: String,
        model: String,
        manufacturer: String,
        // feature info
        feature_uuid: String,
        feature_name: String,
    ) -> Self;
}

impl Sensor for IntSensor {
    fn new(
        // profile info
        profile_owner_id: ObjectId,
        api_token: String,
        // device info
        device_uuid: String,
        mac: String,
        model: String,
        manufacturer: String,
        // feature info
        feature_uuid: String,
        feature_name: String,
    ) -> Self {
        Self::new(
            profile_owner_id,
            api_token,
            device_uuid,
            mac,
            model,
            manufacturer,
            feature_uuid,
            feature_name,
        )
    }
}

impl Sensor for FloatSensor {
    fn new(
        // profile info
        profile_owner_id: ObjectId,
        api_token: String,
        // device info
        device_uuid: String,
        mac: String,
        model: String,
        manufacturer: String,
        // feature info
        feature_uuid: String,
        feature_name: String,
    ) -> Self {
        Self::new(
            profile_owner_id,
            api_token,
            device_uuid,
            mac,
            model,
            manufacturer,
            feature_uuid,
            feature_name,
        )
    }
}

impl IntSensor {
    pub fn new(
        // profile info
        profile_owner_id: ObjectId,
        api_token: String,
        // device info
        device_uuid: String,
        mac: String,
        model: String,
        manufacturer: String,
        // feature info
        feature_uuid: String,
        feature_name: String,
    ) -> Self {
        let date_now: DateTime = DateTime::now();
        Self {
            id: ObjectId::new(),
            profileOwnerId: profile_owner_id,
            apiToken: api_token,
            deviceUuid: device_uuid,
            mac,
            model,
            manufacturer,
            featureUuid: feature_uuid,
            featureName: feature_name,
            value: 0,
            createdAt: date_now,
            modifiedAt: date_now,
        }
    }
}

impl FloatSensor {
    pub fn new(
        // profile info
        profile_owner_id: ObjectId,
        api_token: String,
        // device info
        device_uuid: String,
        mac: String,
        model: String,
        manufacturer: String,
        // feature info
        feature_uuid: String,
        feature_name: String,
    ) -> Self {
        let date_now: DateTime = DateTime::now();
        Self {
            id: ObjectId::new(),
            profileOwnerId: profile_owner_id,
            apiToken: api_token,
            deviceUuid: device_uuid,
            mac,
            model,
            manufacturer,
            featureUuid: feature_uuid,
            featureName: feature_name,
            value: 0.0,
            createdAt: date_now,
            modifiedAt: date_now,
        }
    }
}

pub fn new_from_register_input<T: Sensor + Serialize>(
    input: Json<RegisterInput>,
    sensor_type: &str,
) -> Result<Bson, Error> {
    let profile_owner_id = ObjectId::from_str(input.profileOwnerId.as_str());
    match profile_owner_id {
        Ok(profile_id) => {
            let result = T::new(
                profile_id,
                input.apiToken.clone(),
                input.deviceUuid.clone(),
                input.mac.clone(),
                input.model.clone(),
                input.manufacturer.clone(),
                input.featureUuid.clone(),
                sensor_type.to_string(), // featureName
            );
            Ok(to_bson(&result).unwrap())
        }
        Err(err) => Err(err),
    }
}
