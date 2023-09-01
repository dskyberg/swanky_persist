/// Cache implementation for Redis
use std::sync::Arc;

use redis::{aio::ConnectionManager, AsyncCommands, Client, Value};
use serde::{de::DeserializeOwned, Serialize};

use crate::{Cacheable, DaoError, DaoResult, DataServicesConfig};

#[derive(Clone)]
pub struct Cache {
    pub config: Arc<DataServicesConfig>,
    pub client: Client,
    pub connection_manager: ConnectionManager,
}

impl Cache {
    pub async fn new(config: Arc<DataServicesConfig>) -> DaoResult<Cache> {
        let client = Client::open(config.cache_uri.clone())
            .map_err(|_| DaoError::ServiceError("Redis: Failed to create client".to_string()))?;

        let connection_manager = client.get_tokio_connection_manager().await.map_err(|_| {
            DaoError::ServiceError("Redis: Failed to create connection manager".to_string())
        })?;

        Ok(Self {
            config,
            client,
            connection_manager,
        })
    }

    pub async fn put<T>(&self, value: &T) -> DaoResult<()>
    where
        T: Cacheable + Serialize,
    {
        let cache_key = format!("{}:{}", T::cache_path(), value.cache_id()).to_owned();
        let mut con = self.client.get_async_connection().await?;
        let data = serde_json::to_vec(value)?;
        redis::pipe()
            .atomic()
            .set(&cache_key, data)
            .expire(&cache_key, T::cache_expiry())
            .query_async(&mut con)
            .await?;
        log::trace!("Cached: {}", &cache_key);
        Ok(())
    }

    pub async fn fetch<T>(&self, id: &str) -> DaoResult<Option<T>>
    where
        T: Clone + Cacheable + DeserializeOwned + Unpin + Send + Sync,
    {
        let cache_key = format!("{}:{}", T::cache_path(), id).to_owned();
        let mut con = self.client.get_async_connection().await?;
        let cache_response = con.get(&cache_key).await?;

        match cache_response {
            Value::Nil => {
                log::trace!("Item not in cache: {}", &cache_key);
                Ok(None)
            }
            Value::Data(val) => {
                let result = serde_json::from_slice::<T>(&val)?;
                log::trace!("Fetched from cache: {}", &cache_key);
                Ok(Some(result.clone()))
            }
            _ => Err(DaoError::GeneralError.into()),
        }
    }

    pub async fn delete<T>(&self, id: &str) -> DaoResult<()>
    where
        T: Cacheable,
    {
        let cache_key = format!("{}:{}", T::cache_path(), id).to_owned();
        let mut con = self.client.get_async_connection().await?;
        con.del(&cache_key).await?;
        log::trace!("Deleted from cache: {}", &cache_key);
        Ok(())
    }
}
