use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterInput {
    // profile info
    pub profileOwnerId: String,
    pub apiToken: String,
    // device info
    pub deviceUuid: String,
    pub mac: String,
    pub model: String,
    pub manufacturer: String,
    // feature info
    pub featureUuid: String,
}
