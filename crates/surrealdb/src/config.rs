use std::fmt;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use pbkdf2::pbkdf2_hmac_array;
use serde::Deserialize;
use sha2::Sha256;

const ENCRYPTION_KEY_BYTES: usize = 32;
const PASSPHRASE_PREFIX: &str = "pbkdf2$";

#[derive(Clone, Deserialize)]
pub struct DatabaseConfig {
    pub surreal_url: String,
    pub surreal_namespace: String,
    pub surreal_database: String,
    /// Root username - only used for manual schema migrations via CLI/Surrealist.
    /// The server never uses this at runtime.
    #[serde(default)]
    pub surreal_username: Option<String>,
    /// Root password - only used for manual schema migrations via CLI/Surrealist.
    /// The server never uses this at runtime.
    #[serde(default)]
    pub surreal_password: Option<String>,
    #[serde(default)]
    pub surreal_service_key: String,
    pub surreal_encryption_key: String,
}

#[allow(clippy::missing_errors_doc)]
impl DatabaseConfig {
    pub fn validate(&self) -> Result<(), crate::DatabaseError> {
        if self.surreal_url.trim().is_empty() {
            return Err(crate::DatabaseError::InvalidConfig(
                "surreal_url must not be empty".into(),
            ));
        }

        if self.surreal_namespace.trim().is_empty() {
            return Err(crate::DatabaseError::InvalidConfig(
                "surreal_namespace must not be empty".into(),
            ));
        }

        if self.surreal_database.trim().is_empty() {
            return Err(crate::DatabaseError::InvalidConfig(
                "surreal_database must not be empty".into(),
            ));
        }

        // Note: surreal_username and surreal_password are intentionally NOT validated.
        // They are only needed for manual schema migrations, not for runtime server operation.

        if self.surreal_encryption_key.trim().is_empty() {
            return Err(crate::DatabaseError::InvalidConfig(
                "surreal_encryption_key must not be empty".into(),
            ));
        }

        self.encryption_key_bytes()?;

        Ok(())
    }

    pub fn encryption_key_bytes(&self) -> Result<[u8; ENCRYPTION_KEY_BYTES], crate::DatabaseError> {
        parse_encryption_key(&self.surreal_encryption_key)
    }

    #[must_use]
    pub fn has_service_credentials(&self) -> bool {
        !self.surreal_service_key.trim().is_empty()
    }
}

impl fmt::Debug for DatabaseConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DatabaseConfig")
            .field("surreal_url", &self.surreal_url)
            .field("surreal_namespace", &self.surreal_namespace)
            .field("surreal_database", &self.surreal_database)
            .field("surreal_username", &"[OPTIONAL/REDACTED]")
            .field("surreal_password", &"[OPTIONAL/REDACTED]")
            .field("surreal_service_key", &"[REDACTED]")
            .field("surreal_encryption_key", &"[REDACTED]")
            .finish()
    }
}

fn parse_encryption_key(value: &str) -> Result<[u8; ENCRYPTION_KEY_BYTES], crate::DatabaseError> {
    if value.starts_with(PASSPHRASE_PREFIX) {
        return derive_key_from_passphrase(value);
    }

    if let Ok(decoded) = hex::decode(value) {
        return to_key_array(decoded, "hex");
    }

    let decoded = STANDARD.decode(value).map_err(|_| {
        crate::DatabaseError::InvalidConfig(
            "surreal_encryption_key must be a 32-byte hex value, a 32-byte base64 value, or pbkdf2$<iterations>$<base64-salt>$<passphrase>".into(),
        )
    })?;
    to_key_array(decoded, "base64")
}

fn derive_key_from_passphrase(
    value: &str,
) -> Result<[u8; ENCRYPTION_KEY_BYTES], crate::DatabaseError> {
    let mut parts = value.splitn(4, '$');
    let algorithm = parts.next().unwrap_or_default();
    let iterations = parts
        .next()
        .ok_or_else(invalid_passphrase_format)?
        .parse::<u32>()
        .map_err(|_| invalid_passphrase_format())?;
    let salt = parts.next().ok_or_else(invalid_passphrase_format)?;
    let passphrase = parts.next().ok_or_else(invalid_passphrase_format)?;

    if algorithm != "pbkdf2" || iterations == 0 || passphrase.is_empty() {
        return Err(invalid_passphrase_format());
    }

    let salt = STANDARD
        .decode(salt)
        .map_err(|_| invalid_passphrase_format())?;

    if salt.len() < 16 {
        return Err(crate::DatabaseError::InvalidConfig(
            "surreal_encryption_key passphrase salt must decode to at least 16 bytes".into(),
        ));
    }

    Ok(pbkdf2_hmac_array::<Sha256, ENCRYPTION_KEY_BYTES>(
        passphrase.as_bytes(),
        &salt,
        iterations,
    ))
}

fn to_key_array(
    decoded: Vec<u8>,
    encoding: &str,
) -> Result<[u8; ENCRYPTION_KEY_BYTES], crate::DatabaseError> {
    decoded
        .try_into()
        .map_err(|_| {
            crate::DatabaseError::InvalidConfig(format!(
                "surreal_encryption_key {encoding} value must decode to exactly {ENCRYPTION_KEY_BYTES} bytes"
            ))
        })
}

fn invalid_passphrase_format() -> crate::DatabaseError {
    crate::DatabaseError::InvalidConfig(
        "surreal_encryption_key passphrase format must be pbkdf2$<iterations>$<base64-salt>$<passphrase>".into(),
    )
}

#[cfg(test)]
mod tests {
    use super::DatabaseConfig;

    fn config_with_key(key: &str) -> DatabaseConfig {
        DatabaseConfig {
            surreal_url: "wss://example.test".into(),
            surreal_namespace: "main".into(),
            surreal_database: "main".into(),
            surreal_username: None,
            surreal_password: None,
            surreal_service_key: "service-key".into(),
            surreal_encryption_key: key.into(),
        }
    }

    #[test]
    fn accepts_hex_encryption_key() {
        let config =
            config_with_key("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        let bytes = config.encryption_key_bytes().expect("hex key must decode");
        assert_eq!(bytes[0], 0);
        assert_eq!(bytes[31], 31);
    }

    #[test]
    fn accepts_base64_encryption_key() {
        let config = config_with_key("AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=");
        let bytes = config
            .encryption_key_bytes()
            .expect("base64 key must decode");
        assert_eq!(bytes[0], 0);
        assert_eq!(bytes[31], 31);
    }

    #[test]
    fn accepts_pbkdf2_passphrase_key() {
        let config =
            config_with_key("pbkdf2$1000$MDEyMzQ1Njc4OWFiY2RlZg==$correct horse battery staple");
        let bytes = config
            .encryption_key_bytes()
            .expect("pbkdf2 passphrase must derive");

        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn validates_without_username_password() {
        let config =
            config_with_key("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        assert!(
            config.validate().is_ok(),
            "config should validate without username/password"
        );
    }

    #[test]
    fn allows_missing_service_key() {
        let mut config =
            config_with_key("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        config.surreal_service_key.clear();

        assert!(
            config.validate().is_ok(),
            "missing service key should not block startup"
        );
        assert!(
            !config.has_service_credentials(),
            "blank service key should disable proxy secret fetch"
        );
    }
}
