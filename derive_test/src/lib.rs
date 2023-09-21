//! Demonstrte using the Cache and Persist derive macros for [swanky_persist](../)
//!
use swanky_persist::{Cache, Cacheable, Persist, Persistable};

#[derive(Cache, Persist)]
#[cache(path = "foo")]
#[persist(id_func = self.my_id.clone())]
struct Foo {
    #[cache(id)]
    #[persist(id_field)]
    my_id: String,
}

#[cfg(test)]
mod tests {
    use swanky_persist::{Cache, Cacheable, Persist, Persistable};

    #[test]
    fn test_defaults() {
        #[derive(Persist, Cache)]
        #[persist(name = "foo-collection")]
        #[cache(path = "foo-path")]
        struct Foo {
            // Required field
            id: String,
        }

        let foo = Foo {
            id: "my_id".to_string(),
        };

        assert_eq!(Foo::collection_name(), "foo-collection");
        assert_eq!(Foo::collection_id_field(), "id");
        assert_eq!(foo.collection_id(), "my_id");

        assert_eq!(Foo::cache_path(), "foo-path");
        assert_eq!(Foo::cache_expiry(), 3600);
        assert_eq!(foo.cache_id(), "my_id");
    }

    #[test]
    fn test_different_id_field() {
        #[derive(Persist, Cache)]
        #[persist(name = "foo-collection")]
        #[cache(path = "foo-path")]
        struct Foo {
            #[persist(id, id_field)]
            #[cache(id)]
            _id: String,
        }

        let foo = Foo {
            _id: "my_id".to_string(),
        };

        assert_eq!(foo.collection_id(), "my_id");
        assert_eq!(foo.cache_id(), "my_id");
    }

    #[test]
    fn test_id_func() {
        #[derive(Persist)]
        #[persist(id_func = format!("{}",self.val), id_field = "val")]
        struct Bar {
            val: usize,
        }

        let bar = Bar { val: 24 };
        assert_eq!(bar.collection_id(), "24");
    }

    #[test]
    fn test_cache_expiry() {
        #[derive(Cache)]
        #[cache(expiry = 360)]
        struct Bar {
            #[cache(id)]
            id: String,
        }
        assert_eq!(BAR_CACHE_EXPIRY, 360);
        assert_eq!(Bar::cache_expiry(), 360);
    }
}
