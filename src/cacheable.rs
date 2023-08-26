/// Manages cache details at the object level.
/// Each cacheable object defines its own path into the Redis key namespace as well
/// as it's cache lifetime (in seconds).
/// Each cacheable object instance states it's cache id.
pub trait Cacheable {
    /// The path in Redis to the id
    fn cache_path() -> &'static str;
    /// Each Cacheable object instance provides its own id
    fn cache_id(&self) -> String;
    /// Cache lifetime for this object (in seconds))
    fn cache_expiry() -> usize;
}
