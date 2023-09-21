//! ## Generate the Cacheable trait for a struct
//!
//! ### Struct Attributes
//! * **path:** `String`: The collection name for the struct. Defaults to the struct name.
//! * **expiry** `usize`: The cache expiry time.  Defaults to 3600.
//! * **id_func:** `Expr`: An otional expression to return an id value, if returning a field value is insufficient.
//!
//! ### Field Attributes
//! * **id:** Use this field as the id value returned by `cache_id(&self) -> String`
//!
//! Example
//! ```rust, ignore
//! use swanky_persist::{Cache, Cacheable};
//!
//! #[derive(Cache)]
//! #[cache(path = "foo-cache", expiry = 3600)]
//! struct Foo {
//!     #[cache(id)]
//!     _id: String,
//! }
//!
//! #[derive(Cache)]
//! #[cache(name = "bar-cache", id_func = format!("{}-{}", &self.id, &self.offset))]
//! struct Bar {
//!     id: String,
//!     offset: usize;
//! }
//!
//! #[derive(Cache)]
//! struct FooBar {
//!     id: String,
//! }
//!
//!
//! ```
#![allow(dead_code)]

use darling::{ast, util, FromDeriveInput, FromField};
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Expr, Ident};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(cache), supports(struct_any))]
struct CacheOpts {
    ident: Ident,
    path: Option<String>,
    expiry: Option<usize>,
    id_func: Option<Expr>,
    data: ast::Data<util::Ignored, CacheField>,
}

impl CacheOpts {
    pub fn fields(&self) -> Result<&Vec<CacheField>, String> {
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

    pub fn expiry(&self) -> usize {
        match self.expiry {
            Some(expiry) => expiry,
            None => 3600,
        }
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(cache))]
struct CacheField {
    ident: Option<Ident>,
    #[darling(default)]
    id: bool,
}

impl CacheField {
    pub fn ident(&self) -> String {
        self.ident.as_ref().unwrap().to_string()
    }
}

#[proc_macro_derive(Cache, attributes(cache, cache_id))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let opts = CacheOpts::from_derive_input(&input).expect("Wrong derive options for cache");
    let DeriveInput { ident, .. } = input;

    let id_func = match &opts.id_func {
        Some(id_func) => quote! {
            fn cache_id(&self) -> String {
                #id_func
            }
        },
        None => {
            // No id_func was provided.  So we default to returning the id
            match opts.id() {
                Some(id) => quote! {
                    fn cache_id(&self) -> String {
                        self.#id.clone()
                    }
                },

                None => panic!("#[derive(Cache)] expects an id field or function"),
            }
        }
    };

    let expiry = opts.expiry();
    let cache_expiry_key = format_ident!("{}_CACHE_EXPIRY", ident.to_string().to_uppercase());
    let cache_expiry_const = quote! {
        pub const #cache_expiry_key: usize = #expiry;
    };

    // Set the static str for the collection name field
    let cache_path_key = format_ident!("{}_CACHE_PATH", ident.to_string().to_uppercase());
    let cache_path_const = match opts.path {
        Some(path) => quote! {
            pub const #cache_path_key: &str = #path;
        },
        None => {
            let path = format!("{}", ident.to_string().to_lowercase());
            quote! {
                pub const #cache_path_key: &str = #path;
            }
        }
    };

    let output = quote! {
        #cache_path_const
        #cache_expiry_const
        impl Cacheable for #ident {
            fn cache_path() -> &'static str {
                #cache_path_key
            }
            fn cache_expiry() -> usize {
                #cache_expiry_key
            }
            #id_func
        }
    };
    output.into()
}
