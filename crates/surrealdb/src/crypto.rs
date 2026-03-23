use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::{TryRng, rngs::SysRng};
use sha2::{Digest, Sha256};

use crate::DatabaseError;

const NONCE_SIZE: usize = 12;
const VIRTUAL_API_KEY_BYTES: usize = 32;

pub fn encrypt_secret(
    key_material: &[u8; VIRTUAL_API_KEY_BYTES],
    plaintext: &str,
) -> Result<String, DatabaseError> {
    let cipher = build_cipher(key_material);
    let mut nonce_bytes = [0_u8; NONCE_SIZE];
    SysRng
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|error| DatabaseError::Crypto(format!("failed to generate nonce: {error}")))?;

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_bytes())
        .map_err(|_| DatabaseError::Crypto("failed to encrypt provider secret".into()))?;

    let mut payload = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&ciphertext);

    Ok(STANDARD.encode(payload))
}

pub fn decrypt_secret(
    key_material: &[u8; VIRTUAL_API_KEY_BYTES],
    encoded_payload: &str,
) -> Result<String, DatabaseError> {
    let cipher = build_cipher(key_material);
    let payload = STANDARD
        .decode(encoded_payload)
        .map_err(|_| DatabaseError::Crypto("provider secret is not valid base64".into()))?;

    if payload.len() <= NONCE_SIZE {
        return Err(DatabaseError::Crypto(
            "provider secret payload is too short".into(),
        ));
    }

    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_SIZE);
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| DatabaseError::Crypto("failed to decrypt provider secret".into()))?;

    String::from_utf8(plaintext)
        .map_err(|_| DatabaseError::Crypto("provider secret is not valid utf-8".into()))
}

pub fn generate_virtual_api_key() -> Result<String, DatabaseError> {
    let mut random_bytes = [0_u8; VIRTUAL_API_KEY_BYTES];
    SysRng.try_fill_bytes(&mut random_bytes).map_err(|error| {
        DatabaseError::Crypto(format!("failed to generate API key bytes: {error}"))
    })?;
    Ok(format!("vg_{}", hex::encode(random_bytes)))
}

pub fn hash_virtual_api_key(raw_key: &str) -> String {
    hex::encode(Sha256::digest(raw_key.as_bytes()))
}

pub fn key_prefix(raw_key: &str) -> String {
    raw_key.chars().take(12).collect()
}

fn build_cipher(key_material: &[u8; VIRTUAL_API_KEY_BYTES]) -> Aes256Gcm {
    Aes256Gcm::new_from_slice(key_material).expect("validated encryption key is always 32 bytes")
}

#[cfg(test)]
mod tests {
    use super::{decrypt_secret, encrypt_secret, generate_virtual_api_key, hash_virtual_api_key};
    use pbkdf2::pbkdf2_hmac_array;
    use sha2::Sha256;

    #[test]
    fn provider_secret_round_trip() {
        let key = pbkdf2_hmac_array::<Sha256, 32>(b"test-key", b"0123456789abcdef", 1_000);
        let encrypted = encrypt_secret(&key, "super-secret").expect("encryption must work");
        let decrypted = decrypt_secret(&key, &encrypted).expect("decryption must work");

        assert_eq!(decrypted, "super-secret");
    }

    #[test]
    fn virtual_api_key_hash_is_stable() {
        let raw_key = generate_virtual_api_key().expect("virtual API key generation must work");
        let hash_one = hash_virtual_api_key(&raw_key);
        let hash_two = hash_virtual_api_key(&raw_key);

        assert_eq!(hash_one, hash_two);
        assert_ne!(hash_one, raw_key);
    }
}
