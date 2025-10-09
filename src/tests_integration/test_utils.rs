use rand::prelude::*;

use register::models::inputs::RegisterInput;

pub fn create_register_input(
    profile_owner_id: &str,
    device_uuid: &str,
    mac: &str,
    feature_uuid: &str,
) -> RegisterInput {
    RegisterInput {
        // profile info
        profileOwnerId: profile_owner_id.to_string(),
        apiToken: String::from("473a4861-632b-4915-b01e-cf1d418966c6"),
        // device info
        deviceUuid: device_uuid.to_string(),
        mac: mac.to_string(),
        model: String::from("test-model"),
        manufacturer: String::from("ks89"),
        // feature info
        featureUuid: feature_uuid.to_string(),
    }
}

pub fn build_register_input(profile_owner_id: &str, device_uuid: &str, mac: &str, feature_uuid: &str) -> String {
    serde_json::to_string(&create_register_input(profile_owner_id, device_uuid, mac, feature_uuid)).unwrap()
}

pub fn get_random_mac() -> String {
    const CHARSET: &[u8] = b"ABCDEF0123456789";
    let mut rng = rand::rng();
    let mut mac = String::from("");
    for i in 0..6 {
        let group: String = (0..2)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        if i == 0 {
            mac = format!("{}:", group);
        } else if i > 0 && i < 5 {
            mac = format!("{}{}:", mac, group);
        } else {
            mac = format!("{}{}", mac, group);
        }
    }
    mac
}
