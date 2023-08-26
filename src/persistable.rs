/// Trait for persisting objects in the DB
/// Example,
/// ``` ignore
/// struct MyStruct{
///     my_struct_id: String
/// }
/// impl Persistable for MyStruct {
///     fn collection_name() -> &'static str;{
///         "my_structs"
///     }
///     fn collection_id(&self) -> String {
///         self.my_struct_id.clone()
///     }
///     fn collection_id_field() -> -> &'static str {
///         "my_struct_id"
///     }
/// }
/// ```

pub trait Persistable {
    /// The table name (for RDB) or collection name (for NoSQL) of the data set.
    fn collection_name() -> &'static str;
    /// The id value for a default id based lookup
    fn collection_id(&self) -> String;
    /// The id field to use for default id based lookups.
    fn collection_id_field() -> &'static str {
        "id"
    }
}
