/// Cache abstraction layer
/// The crate currently only supports Redis.  But extending to support other
/// cache services is as simple as adding another target and  then updating the feature flags in
/// [Cargo.toml](./Cargo.toml)
pub use redis_cache::*;

pub mod redis_cache;
