use std::str::FromStr;

use mongodb::bson::oid::ObjectId;
use mongodb::bson::{Bson, DateTime, to_bson};
use serde::{Deserialize, Serialize};

use crate::models::feature_name::FeatureName;
use crate::models::inputs::RegisterInput;
use crate::utils_api_token::encrypt_api_token;
use crate::utils_api_token::hash_api_token;

#[derive(Debug, thiserror::Error)]
pub enum SensorError {
    #[error("invalid profile owner ID: {0}")]
    InvalidObjectId(#[from] mongodb::bson::oid::Error),
    #[error("BSON serialization failed: {0}")]
    BsonSerialize(#[from] mongodb::bson::ser::Error),
    #[error("api token crypto failed: {0}")]
    ApiTokenCrypto(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Sensor<V> {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub profile_owner_id: ObjectId,
    pub api_token_hash: String,
    pub api_token_encrypted: String,
    pub device_uuid: String,
    pub mac: String,
    pub model: String,
    pub manufacturer: String,
    pub feature_uuid: String,
    pub feature_name: String,
    pub value: V,
    pub created_at: DateTime,
    pub modified_at: DateTime,
}

pub type IntSensor = Sensor<i64>;
pub type FloatSensor = Sensor<f64>;

impl<V: Default> Sensor<V> {
    pub fn new(
        profile_owner_id: ObjectId,
        api_token: String,
        device_uuid: String,
        mac: String,
        model: String,
        manufacturer: String,
        feature_uuid: String,
        feature_name: String,
    ) -> Result<Self, SensorError> {
        let now = DateTime::now();
        Ok(Self {
            id: ObjectId::new(),
            profile_owner_id,
            api_token_hash: hash_api_token(&api_token).map_err(SensorError::ApiTokenCrypto)?,
            api_token_encrypted: encrypt_api_token(&api_token).map_err(SensorError::ApiTokenCrypto)?,
            device_uuid,
            mac,
            model,
            manufacturer,
            feature_uuid,
            feature_name,
            value: V::default(),
            created_at: now,
            modified_at: now,
        })
    }
}

impl RegisterInput {
    pub fn into_sensor_bson(self, feature_name: FeatureName) -> Result<Bson, SensorError> {
        let profile_owner_id = ObjectId::from_str(&self.profile_owner_id)?;
        let mac = self.mac.to_ascii_uppercase();
        let feature_name_str = feature_name.to_string();
        let api_token = self.api_token;
        let device_uuid = self.device_uuid;
        let model = self.model;
        let manufacturer = self.manufacturer;
        let feature_uuid = self.feature_uuid;
        let bson = if feature_name.is_float() {
            to_bson(&Sensor::<f64>::new(
                profile_owner_id,
                api_token,
                device_uuid,
                mac,
                model,
                manufacturer,
                feature_uuid,
                feature_name_str,
            )?)?
        } else {
            to_bson(&Sensor::<i64>::new(
                profile_owner_id,
                api_token,
                device_uuid,
                mac,
                model,
                manufacturer,
                feature_uuid,
                feature_name_str,
            )?)?
        };
        Ok(bson)
    }
}
