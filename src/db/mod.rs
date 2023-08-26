/// Data persistence abstraction layer
/// The crate currently only supports Mongodb.  But extending to support other
/// services is as simple as adding another target and then updating the feature flags in
/// [Cargo.toml](./Cargo.toml)
pub use mongo_db::*;

pub mod mongo_db;
