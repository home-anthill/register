use aes_gcm::aead::{Aead, Generate};
use aes_gcm::{Aes256Gcm, KeyInit as AesKeyInit};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use hmac::digest::KeyInit as HmacKeyInit;
use hmac::{Hmac, Mac};
use sha2::Sha256;

const API_TOKEN_NONCE_SIZE: usize = 12;
type ApiTokenNonce = aes_gcm::aead::Nonce<Aes256Gcm>;

pub fn hash_api_token(api_token: &str) -> Result<String, String> {
    let secret = api_token_hash_secret()?;
    let mut mac =
        <Hmac<Sha256> as HmacKeyInit>::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(api_token.as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

pub fn encrypt_api_token(api_token: &str) -> Result<String, String> {
    let cipher =
        <Aes256Gcm as AesKeyInit>::new_from_slice(&api_token_encryption_key()?).map_err(|err| err.to_string())?;
    let nonce = ApiTokenNonce::generate();
    let ciphertext = cipher.encrypt(&nonce, api_token.as_bytes()).map_err(|err| err.to_string())?;
    let mut encoded = nonce.to_vec();
    encoded.extend_from_slice(&ciphertext);
    Ok(URL_SAFE_NO_PAD.encode(encoded))
}

pub fn decrypt_api_token(encrypted: &str) -> Result<String, String> {
    let raw = URL_SAFE_NO_PAD.decode(encrypted).map_err(|err| err.to_string())?;
    if raw.len() <= API_TOKEN_NONCE_SIZE {
        return Err("encrypted api token is too short".to_string());
    }
    let cipher =
        <Aes256Gcm as AesKeyInit>::new_from_slice(&api_token_encryption_key()?).map_err(|err| err.to_string())?;
    let nonce =
        <&ApiTokenNonce>::try_from(&raw[..API_TOKEN_NONCE_SIZE]).map_err(|_| "invalid api token nonce".to_string())?;
    let plaintext = cipher.decrypt(nonce, &raw[API_TOKEN_NONCE_SIZE..]).map_err(|err| err.to_string())?;
    String::from_utf8(plaintext).map_err(|err| err.to_string())
}

fn api_token_hash_secret() -> Result<String, String> {
    dotenvy::dotenv().ok();
    std::env::var("API_TOKEN_HASH_SECRET").map_err(|_| "API_TOKEN_HASH_SECRET is required".to_string())
}

fn api_token_encryption_key() -> Result<[u8; 32], String> {
    dotenvy::dotenv().ok();
    let key =
        std::env::var("API_TOKEN_ENCRYPTION_KEY").map_err(|_| "API_TOKEN_ENCRYPTION_KEY is required".to_string())?;
    if let Ok(decoded) = URL_SAFE_NO_PAD.decode(&key)
        && decoded.len() == 32
    {
        return Ok(decoded.try_into().expect("length checked"));
    }
    if let Ok(decoded) = STANDARD.decode(&key)
        && decoded.len() == 32
    {
        return Ok(decoded.try_into().expect("length checked"));
    }
    if key.len() == 32 {
        return Ok(key.into_bytes().try_into().expect("length checked"));
    }
    Err("API_TOKEN_ENCRYPTION_KEY must be 32 raw bytes or base64-encoded 32 bytes".to_string())
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use hmac::digest::KeyInit as HmacKeyInit;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    use super::{decrypt_api_token, encrypt_api_token, hash_api_token};

    #[test]
    fn hash_api_token_uses_secret_from_env_file() {
        dotenvy::dotenv().ok();
        let secret = std::env::var("API_TOKEN_HASH_SECRET").expect("API_TOKEN_HASH_SECRET is required for tests");
        let api_token = "473a4861-632b-4915-b01e-cf1d418966c6";

        let mut mac =
            <Hmac<Sha256> as HmacKeyInit>::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
        mac.update(api_token.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

        assert_eq!(hash_api_token(api_token).unwrap(), expected);
    }

    #[test]
    fn encrypt_api_token_uses_key_from_env_file() {
        let api_token = "473a4861-632b-4915-b01e-cf1d418966c6";

        let encrypted = encrypt_api_token(api_token).unwrap();

        assert_eq!(decrypt_api_token(&encrypted).unwrap(), api_token);
    }
}
