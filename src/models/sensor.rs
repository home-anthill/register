use std::str::FromStr;

use mongodb::bson::oid::ObjectId;
use mongodb::bson::{oid, to_bson, Bson, DateTime};
use oid::Error;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::models::inputs::RegisterInput;

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntSensor {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub uuid: String,
    pub mac: String,
    pub manufacturer: String,
    pub model: String,
    pub profileOwnerId: ObjectId,
    pub apiToken: String,
    pub createdAt: DateTime,
    pub modifiedAt: DateTime,
    pub value: i64,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FloatSensor {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub uuid: String,
    pub mac: String,
    pub manufacturer: String,
    pub model: String,
    pub profileOwnerId: ObjectId,
    pub apiToken: String,
    pub createdAt: DateTime,
    pub modifiedAt: DateTime,
    pub value: f64,
}

pub trait Sensor {
    fn new(
        uuid: String,
        mac: String,
        manufacturer: String,
        model: String,
        profile_owner_id: ObjectId,
        api_token: String,
    ) -> Self;
}

impl Sensor for IntSensor {
    fn new(
        uuid: String,
        mac: String,
        manufacturer: String,
        model: String,
        profile_owner_id: ObjectId,
        api_token: String,
    ) -> Self {
        Self::new(uuid, mac, manufacturer, model, profile_owner_id, api_token)
    }
}

impl Sensor for FloatSensor {
    fn new(
        uuid: String,
        mac: String,
        manufacturer: String,
        model: String,
        profile_owner_id: ObjectId,
        api_token: String,
    ) -> Self {
        Self::new(uuid, mac, manufacturer, model, profile_owner_id, api_token)
    }
}

impl IntSensor {
    pub fn new(
        uuid: String,
        mac: String,
        manufacturer: String,
        model: String,
        profile_owner_id: ObjectId,
        api_token: String,
    ) -> Self {
        let date_now: DateTime = DateTime::now();
        Self {
            id: ObjectId::new(),
            uuid,
            mac,
            manufacturer,
            model,
            profileOwnerId: profile_owner_id,
            apiToken: api_token,
            createdAt: date_now,
            modifiedAt: date_now,
            value: 0,
        }
    }
}

impl FloatSensor {
    pub fn new(
        uuid: String,
        mac: String,
        manufacturer: String,
        model: String,
        profile_owner_id: ObjectId,
        api_token: String,
    ) -> Self {
        let date_now: DateTime = DateTime::now();
        Self {
            id: ObjectId::new(),
            uuid,
            mac,
            manufacturer,
            model,
            profileOwnerId: profile_owner_id,
            apiToken: api_token,
            createdAt: date_now,
            modifiedAt: date_now,
            value: 0.0,
        }
    }
}

pub fn new_from_register_input<T: Sensor + Serialize>(input: Json<RegisterInput>) -> Result<Bson, Error> {
    let profile_owner_id = ObjectId::from_str(input.profileOwnerId.as_str());
    match profile_owner_id {
        Ok(profile_id) => {
            let result = T::new(
                input.uuid.clone(),
                input.mac.clone(),
                input.manufacturer.clone(),
                input.model.clone(),
                profile_id,
                input.apiToken.clone(),
            );
            Ok(to_bson(&result).unwrap())
        }
        Err(err) => Err(err),
    }
}
