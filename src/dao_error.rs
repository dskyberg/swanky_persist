/// Error management, using [thiserror]
use thiserror::Error;

/// Just re-wrapping for ease of use locally.
pub type DaoResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Error management, using [thiserror]
#[derive(Error, Debug)]
pub enum DaoError {
    #[error("Service error: {0}")]
    ServiceError(String),
    #[error("mongodb error: {0}")]
    DatabaseError(#[from] mongodb::error::Error),
    #[error("could not access field in document: {0}")]
    MongoDataError(#[from] mongodb::bson::document::ValueAccessError),
    #[error("Cache error: {0}")]
    CacheError(#[from] redis::RedisError),
    #[error("A value with this id already exists: {0}")]
    IdExists(String),
    #[error("Not found error")]
    NotFound,
    #[error("General error")]
    GeneralError,
}

impl From<serde_json::Error> for DaoError {
    fn from(_source: serde_json::Error) -> Self {
        Self::GeneralError
    }
}
