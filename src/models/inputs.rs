use std::str::FromStr;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use uuid::{Uuid, Version};

use crate::errors::validation_error::ValidationError;

const MAX_FIELD_LENGTH: usize = 256;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterInput {
    pub profile_owner_id: String,
    pub api_token: String,
    pub device_uuid: String,
    pub mac: String,
    pub model: String,
    pub manufacturer: String,
    pub feature_uuid: String,
}

impl RegisterInput {
    /// Validates all fields, returning an error on failure.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // profileOwnerId must be a valid MongoDB ObjectId
        ObjectId::from_str(&self.profile_owner_id)
            .map_err(|_| ValidationError::new("profileOwnerId must be a valid MongoDB ObjectId"))?;

        // UUID fields: must be valid UUID v4
        validate_uuid_field(&self.device_uuid, "deviceUuid")?;
        validate_uuid_field(&self.feature_uuid, "featureUuid")?;

        // MAC address: XX:XX:XX:XX:XX:XX
        if !is_valid_mac(&self.mac) {
            return Err(ValidationError::new("mac must be in XX:XX:XX:XX:XX:XX format"));
        }

        // apiToken must be a valid UUID v4
        validate_uuid_field(&self.api_token, "apiToken")?;

        // Bounded string fields
        validate_bounded_field(&self.model, "model")?;
        validate_bounded_field(&self.manufacturer, "manufacturer")?;

        Ok(())
    }
}

pub fn validate_uuid_field(value: &str, name: &str) -> Result<(), ValidationError> {
    Uuid::parse_str(value)
        .ok()
        .filter(|u| u.get_version() == Some(Version::Random))
        .map(|_| ())
        .ok_or_else(|| ValidationError::new(format!("{} must be a valid UUID v4", name)))
}

fn validate_bounded_field(value: &str, name: &str) -> Result<(), ValidationError> {
    if value.is_empty() || value.len() > MAX_FIELD_LENGTH {
        return Err(ValidationError::new(format!(
            "{} must be non-empty and at most {} characters",
            name, MAX_FIELD_LENGTH
        )));
    }
    if !value.chars().all(|c| c.is_ascii() && !c.is_ascii_control()) {
        return Err(ValidationError::new(format!("{} contains invalid characters", name)));
    }
    Ok(())
}

fn is_valid_mac(mac: &str) -> bool {
    let mut parts = mac.split(':');
    let valid_count = (&mut parts).take(6).filter(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit())).count();
    valid_count == 6 && parts.next().is_none()
}
