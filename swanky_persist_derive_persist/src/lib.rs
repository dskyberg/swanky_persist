//! ## Generate the Derivable trate for a struct
//! ### Struct Attributes
//! * **name:** `String`: The collection name for the struct. This attribute is required.
//! * **id_func:** `Expr`: An otional expression to return an id value. This will override the `id` attribute above.
//! * **id_field:** `String`: The search key in the collection. Use this if the search key is different from the name of a field.
//!
//! ### Field Attributes
//! * **id:** Use this field as the id value returned by `collection_id(&self) -> String`
//! * **id_field:** Use this field name as the search key  returned by `collection_id_field() -> String`
//!
//! Example
//! ```rust, ignore
//! use swanky_persist::{Persist, Persistable};
//!
//! #[derive(Persist, Cache)]
//! #[persist(name = "foo-collection")]
//! struct Foo {
//!     #[persist(id, id_field)]
//!     _id: String,
//! }
//!
//! #[derive(Persist, Cache)]
//! #[persist(name = "bar-collection", id_func = format!("{}-{}", &self.id, &self.offset))]
//! struct Bar {
//!     id: String,
//!     offset: usize;
//! }
//! ```
#![allow(dead_code)]

use darling::{ast, util, FromDeriveInput, FromField};
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Expr, Ident};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(persist), supports(struct_any))]
struct PersistOpts {
    pub name: Option<String>,
    pub id_func: Option<Expr>,
    pub id_field: Option<String>,
    data: ast::Data<util::Ignored, PersistField>,
}

impl PersistOpts {
    pub fn fields(&self) -> Result<&Vec<PersistField>, String> {
        match &self.data {
            ast::Data::Struct(fields) => Ok(&fields.fields),

            _ => Err("not a struct".to_string()),
        }
    }

    /// Look for a field that has the id attribute set.
    /// If not found, look for a field called "id".
    /// If neither are found, return None.
    pub fn id(&self) -> Option<&Ident> {
        let mut id_ident: Option<&Ident> = None;

        for field in self.fields().unwrap() {
            if field.id {
                return Some(field.ident.as_ref().unwrap());
            }
            let f = field.ident.as_ref().unwrap();
            if f.to_string() == "id" {
                id_ident = Some(field.ident.as_ref().unwrap());
            }
        }
        id_ident
    }

    /// Look for a struct attribute for id_field.
    /// If not found, look for a field with `#[persist(id_field )].
    /// If not found, look for a field named `id`.
    /// If none of the above are found, return None.
    pub fn id_field(&self) -> Option<Ident> {
        let mut id_ident: Option<Ident> = None;
        if self.id_field.is_some() {
            return Some(format_ident!("{}", self.id_field.as_ref().unwrap()));
        }
        for field in self.fields().unwrap() {
            if field.id_field {
                return Some(field.ident.as_ref().unwrap().clone());
            }
            let f = field.ident.as_ref().unwrap();
            if f.to_string() == "id" {
                id_ident = Some(field.ident.as_ref().unwrap().clone());
            }
        }
        id_ident
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(persist))]
struct PersistField {
    ident: Option<Ident>,
    #[darling(default)]
    id: bool,
    #[darling(default)]
    id_field: bool,
}

impl PersistField {
    pub fn ident(&self) -> String {
        self.ident.as_ref().unwrap().to_string()
    }
}

#[proc_macro_derive(Persist, attributes(persist))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);

    let opts = PersistOpts::from_derive_input(&input).expect("Wrong persist options");
    let DeriveInput { ident, .. } = input;

    // If an id function was provided, then use that.

    let id_func = match &opts.id_func {
        Some(id_func) => quote! {
            fn collection_id(&self) -> String {
                #id_func
            }
        },
        None => {
            // No id_func was provided.  So we default to returning the id
            match opts.id() {
                Some(id) => quote! {
                    fn collection_id(&self) -> String {
                        self.#id.clone()
                    }
                },
                None => quote! {
                    fn collection_id(&self) -> String {
                        self.id.clone()
                    }
                },
            }
        }
    };

    // Set the static str for the collection name field
    let collection_name_key = format_ident!("{}_COLLECTION_NAME", ident.to_string().to_uppercase());
    // Set the static str for the collection name field
    let collection_id_field_key =
        format_ident!("{}_COLLECTION_ID_FIELD", ident.to_string().to_uppercase());

    let collection_name_const = match &opts.name {
        Some(name) => quote! {
            pub const #collection_name_key: &str = #name;
        },
        None => {
            let name = format!("{}", ident.to_string().to_lowercase());
            quote! {
                pub const #collection_name_key: &str = #name;
            }
        }
    };

    // Set the static str for the id field
    let id_field_const = match &opts.id_field() {
        Some(id) => {
            let id = format!("{}", id.to_string());
            quote! {
                pub const #collection_id_field_key: &str = #id;
            }
        }
        None => quote! {
            pub const #collection_id_field_key: &str = "id";
        },
    };

    let output = quote! {
        #collection_name_const
        #id_field_const
        impl Persistable for #ident {
            fn collection_name() -> &'static str {
                #collection_name_key
            }
            #id_func
            fn collection_id_field() -> &'static str {
                #collection_id_field_key
            }
        }
    };
    output.into()
}
