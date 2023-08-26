use std::env;

use super::DaoResult;

#[derive(Debug, Clone)]
pub struct DataServicesConfig {
    pub db_database: String,
    pub db_app_name: String,
    pub db_uri: String,
    pub cache_uri: String,
}

impl DataServicesConfig {
    pub fn new() -> DaoResult<Self> {
        let db_database = env::var("SWANKY_DB_DATABASE").map_err(|e| {
            log::error!("SWANKY_DB_DATABASE was not set");
            e
        })?;
        let db_app_name = env::var("SWANKY_DB_APP_NAME").map_err(|e| {
            log::error!("SWANKY_DB_APP_NAME was not set");
            e
        })?;
        let db_uri = env::var("SWANKY_DB_URI").map_err(|e| {
            log::error!("SWANKY_DB_URI was not set");
            e
        })?;
        let cache_uri = env::var("SWANKY_CACHE_URI").map_err(|e| {
            log::error!("SWANKY_CACHE_URI was not set");
            e
        })?;

        Ok(Self {
            db_database,
            db_app_name,
            db_uri,
            cache_uri,
        })
    }
}
