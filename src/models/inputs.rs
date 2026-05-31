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

#[cfg(test)]
mod tests {
    use super::RegisterInput;

    fn valid_input() -> RegisterInput {
        RegisterInput {
            profile_owner_id: "63963ce7c7fd6d463c6c77a3".to_string(),
            api_token: "473a4861-632b-4915-b01e-cf1d418966c6".to_string(),
            device_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            mac: "AA:BB:CC:DD:EE:FF".to_string(),
            model: "test-model".to_string(),
            manufacturer: "ks89".to_string(),
            feature_uuid: "6f8b59c2-4ed4-4419-8f66-a59e992ebb54".to_string(),
        }
    }

    #[test]
    fn validate_accepts_valid_input() {
        let input = valid_input();

        assert!(input.validate().is_ok());
    }

    #[test]
    fn validate_rejects_invalid_device_uuid() {
        let mut input = valid_input();
        input.device_uuid = "not-a-uuid".to_string();

        let err = input.validate().expect_err("invalid device UUID must fail");

        assert_eq!(err.message, "deviceUuid must be a valid UUID v4");
    }

    #[test]
    fn validate_rejects_non_v4_feature_uuid() {
        let mut input = valid_input();
        input.feature_uuid = "12345678-1234-1234-1234-123456789012".to_string();

        let err = input.validate().expect_err("non-v4 feature UUID must fail");

        assert_eq!(err.message, "featureUuid must be a valid UUID v4");
    }

    #[test]
    fn validate_rejects_malformed_mac() {
        for mac in ["AA:BB:CC:DD:EE", "AA:BB:CC:DD:EE:GG", "AA-BB-CC-DD-EE-FF"] {
            let mut input = valid_input();
            input.mac = mac.to_string();

            let err = input.validate().expect_err("malformed MAC must fail");

            assert_eq!(err.message, "mac must be in XX:XX:XX:XX:XX:XX format");
        }
    }

    #[test]
    fn validate_rejects_empty_model() {
        let mut input = valid_input();
        input.model.clear();

        let err = input.validate().expect_err("empty model must fail");

        assert_eq!(err.message, "model must be non-empty and at most 256 characters");
    }

    #[test]
    fn validate_rejects_too_long_manufacturer() {
        let mut input = valid_input();
        input.manufacturer = "a".repeat(257);

        let err = input.validate().expect_err("too long manufacturer must fail");

        assert_eq!(err.message, "manufacturer must be non-empty and at most 256 characters");
    }

    #[test]
    fn validate_rejects_non_ascii_model() {
        let mut input = valid_input();
        input.model = "model-\u{00e9}".to_string();

        let err = input.validate().expect_err("non-ASCII model must fail");

        assert_eq!(err.message, "model contains invalid characters");
    }

    #[test]
    fn validate_rejects_control_character_in_manufacturer() {
        let mut input = valid_input();
        input.manufacturer = "ks89\nlabs".to_string();

        let err = input.validate().expect_err("control character must fail");

        assert_eq!(err.message, "manufacturer contains invalid characters");
    }
}
