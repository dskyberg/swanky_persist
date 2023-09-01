#![ doc=include_str!( "../README.md")]
//! # Example
//! ```
//! # tokio_test::block_on(async {
//!  use std::sync::Arc;
//!  use std::env;
//!  use swanky_persist::*;
//!
//!  env::set_var("SWANKY_DB_DATABASE", "demo");
//!  env::set_var("SWANKY_DB_APP_NAME", "demo");
//!  env::set_var("SWANKY_DB_URI", "mongodb://127.0.0.1:27017");
//!  env::set_var("SWANKY_CACHE_URI", "redis://127.0.0.1");
//!
//!  #[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq)]
//!  struct DemoStruct {
//!      id: String,
//!      name: String,
//!      description: String,
//!  }

//!  const DEMO_STRUCT: &str = "demo_struct";
//!  const DEMO_STRUCT_ID_FIELD: &str = "id";
//!
//!  impl Cacheable for DemoStruct {
//!      fn cache_expiry() -> usize {
//!          3600
//!      }
//!      fn cache_id(&self) -> String {
//!          self.id.clone()
//!      }
//!      fn cache_path() -> &'static str {
//!          DEMO_STRUCT
//!      }
//!  }
//!
//!  impl Persistable for DemoStruct {
//!      fn collection_id(&self) -> String {
//!          self.id.clone()
//!      }
//!      fn collection_id_field() -> &'static str {
//!          DEMO_STRUCT_ID_FIELD
//!      }
//!      fn collection_name() -> &'static str {
//!          DEMO_STRUCT
//!      }
//!  }
//!
//!  // Starting instance for the test
//!  let demo_struct = DemoStruct {
//!      id: String::from("id_1"),
//!      name: String::from("Demo Struct 1"),
//!      description: String::from("description of this obj"),
//!  };
//!
//!  let data_services_config =
//!      Arc::new(DataServicesConfig::new().expect("Failed to create DataServicesConfig"));
//!
//!  let services = DataServices::new(data_services_config.clone())
//!     .await
//!      .expect("Services failed. Did you remember to start them?");
//!
//!  // Insert the instance and varify it's there
//!  let result = services
//!      .add_cached(demo_struct.clone())
//!      .await
//!      .expect("Failed to store the object");
//!  assert_eq!(&demo_struct, &result);
//!
//!  let result = services
//!      .fetch_cached::<DemoStruct>(&result.id)
//!      .await
//!      .expect("Failed to fetch the object")
//!      .unwrap();
//!  assert_eq!(&demo_struct, &result);
//!
//!  // Update the object and verify it changed
//!  let new_obj = services
//!      .update_cached::<DemoStruct, String>(
//!          &result.collection_id(),
//!          "description",
//!          "Updated description of this obj".to_string(),
//!      )
//!      .await
//!      .expect("Failed to update the object")
//!     .unwrap();
//!  assert_eq!(&new_obj.description, "Updated description of this obj");
//!
// Delete the object and verify it's gone
//!  services
//!      .delete_cached::<DemoStruct>(&new_obj.collection_id())
//!      .await
//!      .expect("Failed to delete object");
//!
//!  let result = services
//!      .fetch_cached::<DemoStruct>(&new_obj.collection_id())
//!      .await
//!     .expect("Failed to fetch object again");
//!  assert!(result.is_none());
//! # })
//! ```
pub use cache::redis_cache::*;
pub use cacheable::*;
pub use dao_error::*;
pub use data_services::*;
pub use data_services_config::*;
pub use db::mongo_db::*;
pub use persistable::*;

mod cache;
mod cacheable;
mod dao_error;
mod data_services;
mod data_services_config;
mod db;
mod persistable;
