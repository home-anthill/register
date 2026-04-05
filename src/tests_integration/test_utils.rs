use rand::RngExt;

use register::models::inputs::RegisterInput;

pub fn create_register_input(
    profile_owner_id: &str,
    device_uuid: &str,
    mac: &str,
    feature_uuid: &str,
) -> RegisterInput {
    RegisterInput {
        profile_owner_id: profile_owner_id.to_string(),
        api_token: String::from("473a4861-632b-4915-b01e-cf1d418966c6"),
        device_uuid: device_uuid.to_string(),
        mac: mac.to_string(),
        model: String::from("test-model"),
        manufacturer: String::from("ks89"),
        feature_uuid: feature_uuid.to_string(),
    }
}

pub fn build_register_input(profile_owner_id: &str, device_uuid: &str, mac: &str, feature_uuid: &str) -> String {
    serde_json::to_string(&create_register_input(profile_owner_id, device_uuid, mac, feature_uuid)).unwrap()
}

pub fn build_register_input_with_token(
    profile_owner_id: &str,
    device_uuid: &str,
    mac: &str,
    feature_uuid: &str,
    api_token: &str,
) -> String {
    let mut input = create_register_input(profile_owner_id, device_uuid, mac, feature_uuid);
    input.api_token = api_token.to_string();
    serde_json::to_string(&input).unwrap()
}

pub fn get_random_mac() -> String {
    const CHARSET: &[u8] = b"ABCDEF0123456789";
    let mut rng = rand::rng();
    let groups: Vec<String> =
        (0..6).map(|_| (0..2).map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char).collect()).collect();
    groups.join(":")
}
