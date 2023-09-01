use futures::stream::TryStreamExt;
use std::sync::Arc;

use mongodb::{
    bson::{doc, Bson},
    options::ClientOptions,
    Client, Database,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{DaoError, DaoResult, DataServicesConfig, Persistable};

#[derive(Clone, Debug)]
pub struct DB {
    pub config: Arc<DataServicesConfig>,
    pub client: Client,
    pub database: Database,
}

impl DB {
    pub async fn new(config: Arc<DataServicesConfig>) -> DaoResult<Self> {
        // Create the ClientOptions and set the app_name
        let mut client_options = ClientOptions::parse(&config.db_uri).await.map_err(|_| {
            DaoError::ServiceError("MongoDB: failed to parse client options".to_string())
        })?;
        client_options.app_name = Some(config.db_app_name.clone());

        // Create the client and grab a database handle
        let client = Client::with_options(client_options)
            .map_err(|_| DaoError::ServiceError("Failed to create MongoDB client".to_string()))?;
        let database = client.database(&config.db_database);
        Ok(Self {
            config,
            client,
            database,
        })
    }

    pub async fn add<T>(&self, value: T) -> DaoResult<T>
    where
        T: core::fmt::Debug
            + Clone
            + Send
            + Sync
            + Unpin
            + DeserializeOwned
            + Serialize
            + Persistable,
    {
        let collection_name = T::collection_name();

        let collection = self.database.collection::<T>(collection_name);
        match self.fetch_by_id::<T>(&value.collection_id()).await? {
            Some(_) => Err(DaoError::IdExists(value.collection_id()).into()),
            None => match collection.insert_one(&value, None).await {
                Ok(_) => {
                    log::trace!("Added {}: {}", collection_name, value.collection_id());
                    Ok(value)
                }
                Err(err) => {
                    log::error!("Error saving {}: {:?}", collection_name, &err);
                    Err(DaoError::DatabaseError(err).into())
                }
            },
        }
    }

    pub async fn fetch<T, K>(
        &self,
        key: Option<&str>,
        value: Option<K>,
    ) -> DaoResult<Option<Vec<T>>>
    where
        T: Clone + DeserializeOwned + Unpin + Send + Sync + Persistable,
        K: Serialize,
    {
        let collection_name = T::collection_name();
        let filter = match (key, value) {
            (Some(k), Some(v)) => doc! {k: serde_json::to_string(&v)?},
            _ => doc! {},
        };

        let cursor_result = self
            .database
            .collection::<T>(collection_name)
            .find(filter, None)
            .await
            .map_err(DaoError::DatabaseError);

        match cursor_result {
            Ok(cursor) => {
                let result = cursor.try_collect::<Vec<T>>().await?;
                if result.is_empty() {
                    return Ok(None);
                }
                Ok(Some(result))
            }
            Err(e) => {
                log::trace!("fetch returned en error: {:?}", e);
                Err(e.into())
            }
        }
    }

    pub async fn fetch_by_id<T>(&self, id: &str) -> DaoResult<Option<T>>
    where
        T: Clone + DeserializeOwned + Unpin + Send + Sync + Persistable,
    {
        let collection_name = T::collection_name();
        let filter = doc! {T::collection_id_field(): id.to_string()};
        let cursor_result = self
            .database
            .collection::<T>(collection_name)
            .find_one(filter, None)
            .await
            .map_err(DaoError::DatabaseError);
        match cursor_result {
            Ok(cursor) => match cursor {
                Some(result) => {
                    log::trace!(
                        "Fetched {} - {}:{}",
                        collection_name,
                        T::collection_id_field(),
                        id
                    );
                    Ok(Some(result))
                }
                None => {
                    log::trace!(
                        "Fetch not found: {} - {}:{}",
                        collection_name,
                        T::collection_id_field(),
                        id
                    );
                    Ok(None)
                }
            },
            Err(e) => {
                log::trace!("fetch_by_id returned en error: {:?}", e);
                Err(e.into())
            }
        }
    }

    /// Persisted objects are essentially  hierarchical key value stores.  Let's start with the
    /// top level objects, since we will be micro adjusting them.  So, update needs:
    /// - the object id
    /// - the field name of the value being updated
    /// - the new value for that field
    pub async fn update<T, K>(&self, id: &str, key: &str, value: K) -> DaoResult<Option<T>>
    where
        T: Clone + DeserializeOwned + Unpin + Send + Sync + Persistable,
        K: Clone + Serialize + Into<Bson>,
    {
        let collection_name = T::collection_name();
        let collection = self.database.collection::<T>(collection_name);

        let filter = doc! {T::collection_id_field(): &id.to_string()};

        let set = doc! {"$set": doc! {key: &value.into()}};

        match collection.update_one(filter, set, None).await {
            Ok(res) => {
                log::trace!("Updated {}: {} - {:?}", collection_name, id, &res);
                self.fetch_by_id::<T>(id).await
            }
            Err(err) => {
                log::error!("Error updating {}: {:?}", collection_name, &err);
                Err(DaoError::DatabaseError(err).into())
            }
        }
    }

    pub async fn delete<T>(&self, id: &str) -> DaoResult<()>
    where
        T: Persistable,
    {
        let collection_name = T::collection_name();
        let filter = doc! {T::collection_id_field(): &id.to_string()};
        self.database
            .collection::<T>(collection_name)
            .delete_one(filter, None)
            .await
            .map_err(DaoError::DatabaseError)?;
        log::trace!(
            "Deleted {} - {}:{}",
            collection_name,
            T::collection_id_field(),
            id
        );

        Ok(())
    }
}
