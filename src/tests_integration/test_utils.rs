use rand::Rng;

use register::models::inputs::RegisterInput;

pub fn create_register_input(sensor_uuid: &str, mac: &str, profile_owner_id: &str) -> RegisterInput {
    RegisterInput {
        uuid: sensor_uuid.to_string(),
        mac: mac.to_string(),
        manufacturer: String::from("ks89"),
        model: String::from("test-model"),
        profileOwnerId: profile_owner_id.to_string(),
        apiToken: String::from("473a4861-632b-4915-b01e-cf1d418966c6"),
    }
}

pub fn build_register_input(sensor_uuid: &str, mac: &str, profile_owner_id: &str) -> String {
    serde_json::to_string(&create_register_input(sensor_uuid, mac, profile_owner_id)).unwrap()
}

pub fn get_random_mac() -> String {
    const CHARSET: &[u8] = b"ABCDEF0123456789";
    let mut rng = rand::thread_rng();
    let mut mac = String::from("");
    for i in 0..6 {
        let group: String = (0..2)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
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
