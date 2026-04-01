use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("invalid database configuration: {0}")]
    InvalidConfig(String),

    #[error("database service authentication failed: {0}")]
    ServiceAuth(String),

    #[error("provider secret fetch failed: {0}")]
    SecretFetch(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("database operation failed: {0}")]
    Database(#[from] Box<surrealdb::Error>),

    #[error("schema bootstrap failed: {0}")]
    SchemaBootstrap(String),

    #[error("cryptography error: {0}")]
    Crypto(String),
}

impl From<surrealdb::Error> for DatabaseError {
    fn from(error: surrealdb::Error) -> Self {
        Self::Database(Box::new(error))
    }
}
