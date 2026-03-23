use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub surreal_url: String,
    pub surreal_namespace: String,
    pub surreal_database: String,
    pub surreal_username: String,
    pub surreal_password: String,
    pub surreal_encryption_key: String,
}

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

        if self.surreal_username.trim().is_empty() {
            return Err(crate::DatabaseError::InvalidConfig(
                "surreal_username must not be empty".into(),
            ));
        }

        if self.surreal_password.trim().is_empty() {
            return Err(crate::DatabaseError::InvalidConfig(
                "surreal_password must not be empty".into(),
            ));
        }

        if self.surreal_encryption_key.trim().is_empty() {
            return Err(crate::DatabaseError::InvalidConfig(
                "surreal_encryption_key must not be empty".into(),
            ));
        }

        Ok(())
    }
}
