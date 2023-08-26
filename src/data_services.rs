/// Persistance layer.  This layer doesn't much care about what fulfills the Cache and DB layers.
/// One expection is that MongoDB requires anythiung added to a Document to implement
/// Into<mongodb::bson::Bson>. While you don't have to implement that for your structs, it does have
/// to be declared as a trrait on the `modify` methods.  If anyone can figure out how I can
/// abstract to just use serde traits, that would be awesome!
use serde::{de::DeserializeOwned, Serialize};

use super::{Cache, Cacheable, DaoResult, DataServicesConfig, Persistable, DB};

#[derive(Clone)]
pub struct DataServices<'a> {
    pub config: &'a DataServicesConfig,
    /// Represents the Redis cache client
    pub cache: Cache<'a>,
    pub db: DB,
}

#[allow(dead_code)]
impl<'a> DataServices<'a> {
    /// Establishes the client connections to the database and cache.
    ///
    /// This should be called only once in the crate main.
    pub async fn new(config: &'a DataServicesConfig) -> DaoResult<DataServices<'a>> {
        let cache = Cache::new(config).await?;
        let db = DB::new(config).await?;
        Ok(DataServices { config, cache, db })
    }

    /// Add an object instance to the DB
    pub async fn add<T>(&self, value: T) -> DaoResult<T>
    where
        T: core::fmt::Debug + Clone + Serialize + Persistable,
    {
        self.db.add(value).await
    }

    /// Add an object to the db and cache it
    pub async fn add_cached<T>(&self, value: T) -> DaoResult<T>
    where
        T: Persistable + Cacheable + std::fmt::Debug + std::clone::Clone + Serialize,
    {
        let result = self.db.add(value).await?;
        self.cache.put(&result).await?;
        Ok(result)
    }

    /// Fetch an object.
    /// This fetches straight from the db.  No cache involved.
    pub async fn fetch<T>(&self, id: &str) -> DaoResult<Option<T>>
    where
        T: Clone + DeserializeOwned + Unpin + Send + Sync + Persistable,
    {
        self.db.fetch_by_id::<T>(id).await
    }

    /// Fetch a possibly cached object.
    /// Looks in cache first.  If not found, it looks in DB.  If found, it adds t
    /// the cache.
    pub async fn fetch_cached<T>(&self, id: &str) -> DaoResult<Option<T>>
    where
        T: Clone + Persistable + Cacheable + DeserializeOwned + Serialize + Unpin + Send + Sync,
    {
        match self.cache.fetch::<T>(id).await? {
            Some(t) => Ok(Some(t)),
            None => {
                // The item is not in cache.  Look in the db.
                let result = self.db.fetch_by_id::<T>(id).await?;
                match result {
                    Some(t) => {
                        // Found the object in the db.  So cache it and then return it
                        self.cache.put(&t).await?;
                        Ok(Some(t))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    /// Update a persisted object.
    pub async fn update<T, K>(&self, id: &str, key: &str, value: K) -> DaoResult<Option<T>>
    where
        T: Clone + DeserializeOwned + Unpin + Send + Sync + Persistable,
        K: Clone + Serialize + Into<mongodb::bson::Bson>, // mongodb::bson::Bson: std::convert::From<K>,
    {
        self.db.update::<T, K>(id, key, value).await
    }

    /// Update a persisted object, and refresh the cache.
    /// We just re-put the object in the cache, so that expiry times are updated appropriately.
    pub async fn update_cached<T, K>(&self, id: &str, key: &str, value: K) -> DaoResult<Option<T>>
    where
        T: Clone + Persistable + Cacheable + DeserializeOwned + Serialize + Unpin + Send + Sync,
        K: Clone + Serialize + Into<mongodb::bson::Bson>,
    {
        match self.db.update::<T, K>(id, key, value).await? {
            Some(object) => {
                self.cache.put::<T>(&object).await?;
                Ok(Some(object))
            }
            None => Ok(None),
        }
    }

    /// Delete an object from the db.
    /// Note, if you cached the object, and are calling this, your cachee will not match the db. use [DataServices::delete_cached] instead.
    pub async fn delete<T>(&self, id: &str) -> DaoResult<()>
    where
        T: Persistable,
    {
        self.db.delete::<T>(id).await
    }

    /// Delete an object from both the db and the cache.
    pub async fn delete_cached<T>(&self, id: &str) -> DaoResult<()>
    where
        T: Persistable + Cacheable,
    {
        self.db.delete::<T>(id).await?;
        self.cache.delete::<T>(id).await?;
        Ok(())
    }
}
