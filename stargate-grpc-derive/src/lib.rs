//! # Derive macros for mapping between `Value` and Rust structs
//!
//! Converting structures from/to hash maps manually is tedious.
//! This module defines a few derive macros that can generate the conversion code automatically
//! for you.
//!
//! ## Converting a custom Rust struct to a `Value`
//! ```
//! use stargate_grpc::Value;
//! use stargate_grpc_derive::IntoValue;
//! #[derive(IntoValue)]
//! struct User {
//!     id: i64,
//!     login: &'static str
//! }
//!
//! let user = User { id: 1, login: "user" };
//! let value = Value::from(user);
//!
//! assert_eq!(value, Value::udt(vec![("id", Value::bigint(1)), ("login", Value::string("user"))]))
//! ```
//!
//! ## Converting a `Value` to a custom Rust struct
//! ```
//! use stargate_grpc::Value;
//! use stargate_grpc_derive::TryFromValue;
//!
//! #[derive(TryFromValue)]
//! struct User {
//!     id: i64,
//!     login: String
//! }
//!
//! let value = Value::udt(vec![("id", Value::bigint(1)), ("login", Value::string("user"))]);
//! let user: User = value.try_into().unwrap();
//!
//! assert_eq!(user.id, 1);
//! assert_eq!(user.login, "user".to_string());
//! ```
//!
//! ## Using custom structs as arguments in queries
//! It is possible to unpack struct fields in such a way that each field value
//! gets bound to a named argument of a query. For that to work, the struct must implement
//! [`std::convert::Into<Values>`] trait. You can derive such trait automatically:
//!
//! ```
//! use stargate_grpc::Query;
//! use stargate_grpc_derive::IntoValues;
//!
//! #[derive(IntoValues)]
//! struct User {
//!     id: i64,
//!     login: &'static str
//! }
//!
//! let user = User { id: 1, login: "user" };
//! let query = Query::builder()
//!     .query("INSERT INTO users(id, login) VALUES (:id, :login)")
//!     .bind(user)  // bind user.id to :id and user.login to :login
//!     .build();
//! ```
//! ## Converting result set rows to custom struct values
//! You can convert a `Row` to a value of your custom type by deriving
//! [`TryFromRow`] and then passing the rows to a mapper:
//!
//! ```no_run
//! use stargate_grpc::*;
//! use stargate_grpc_derive::*;
//!
//! #[derive(Debug, TryFromRow)]
//! struct User {
//!     id: i64,
//!     login: String,
//! }
//!
//! let result_set: ResultSet = unimplemented!();  // replace with actual code to run a query
//! let mapper = result_set.mapper().unwrap();
//! for row in result_set.rows {
//!     let user: User = mapper.try_unpack(row).unwrap();
//!     println!("{:?}", user)
//! }
//!
//! ```
//!
//! ## Options
//! All macros defined in this module accept a `#[stargate]` attribute that you can set
//! on struct fields to control the details of how the conversion should be made.
//!
//! ### `#[stargate(skip)]`
//! Skips the field when doing the conversion to `Value`. This is useful when the structure
//! needs to store some data that are not mapped to the database schema.
//! However, the field is included in the conversion from `Value`, and the conversion would fail
//! if it was missing, hence you likely need to set `#[stargate(default)]` as well.
//!
//! ### `#[stargate(default)]`
//! Uses the default value for the field type provided by [`std::default::Default`],
//! if the source `Value` doesn't contain the field, or if the field is set to `Value::null`
//! or `Value::unset`.
//!
//! ### `#[stargate(default = "expression")]`
//! Obtains the default value by evaluating given Rust expression given as a string.
//!
//! ```
//! use stargate_grpc_derive::TryFromValue;
//!
//! fn default_file_name() -> String {
//!     "file.txt".to_string()
//! }
//!
//! #[derive(TryFromValue)]
//! struct File {
//!     #[stargate(default = "default_file_name()")]
//!     path: String,
//! }
//! ```
//!
//! ### `#[stargate(cql_type = "type")]`
//! Sets the target CQL type the field should be converted into, useful
//! when there are multiple possibilities.
//!
//! ```
//! use stargate_grpc::types;
//! use stargate_grpc_derive::IntoValue;
//!
//! #[derive(IntoValue)]
//! struct InetAndUuid {
//!     #[stargate(cql_type = "types::Inet")]
//!     inet: [u8; 16],
//!     #[stargate(cql_type = "types::Uuid")]
//!     uuid: [u8; 16],
//! }
//! ```
//!
//! ### `#[stargate(name = "column")]`
//! Sets the CQL field, column or query argument name associated with the field.
//! If not given, it is assumed to be the same as struct field name.
//!
use proc_macro::TokenStream;

use darling::util::Override;
use darling::{ast, util, FromDeriveInput, FromField};
use quote::quote;
use syn::__private::TokenStream2;

// Verify examples in the readme:
#[doc = include_str!("../README.md")]
type _DoctestReadme = ();

#[derive(Debug, FromField)]
#[darling(attributes(stargate))]
struct UdtField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    default: Option<Override<String>>,
    #[darling(default)]
    cql_type: Option<String>,
    #[darling(default)]
    skip: bool,
    #[darling(default)]
    name: Option<String>,
}

#[derive(Debug, FromDeriveInput)]
struct Udt {
    ident: syn::Ident,
    data: ast::Data<util::Ignored, UdtField>,
}

fn get_fields(udt: ast::Data<util::Ignored, UdtField>) -> Vec<UdtField> {
    match udt {
        ast::Data::Struct(s) => s.fields,
        _ => panic!("Deriving IntoValue allowed only on structs"),
    }
}

fn field_idents(fields: &[UdtField]) -> Vec<&syn::Ident> {
    fields.iter().map(|f| f.ident.as_ref().unwrap()).collect()
}

/// Lists the field names of the associated Udt, Row or Values.
fn field_names(fields: &[UdtField]) -> Vec<String> {
    fields
        .iter()
        .map(|f| {
            f.name
                .clone()
                .unwrap_or_else(|| f.ident.as_ref().unwrap().to_string())
        })
        .collect()
}

fn token_stream(s: &str) -> proc_macro2::TokenStream {
    s.parse().unwrap()
}

/// Emits code for reading the field value and converting it to a `Value`.
fn convert_to_value(obj: &syn::Ident, field: &UdtField) -> TokenStream2 {
    let field_ident = field.ident.as_ref().unwrap();
    match &field.cql_type {
        Some(t) => {
            let cql_type = token_stream(t.as_str());
            quote! { stargate_grpc::Value::of_type(#cql_type, #obj.#field_ident) }
        }
        None => {
            quote! { stargate_grpc::Value::from(#obj.#field_ident) }
        }
    }
}

/// For each field, returns an expression that converts that field's value to a `Value`.
fn convert_to_values(obj: &syn::Ident, fields: &[UdtField]) -> Vec<TokenStream2> {
    fields.iter().map(|f| convert_to_value(obj, f)).collect()
}

/// Derives the `IntoValue` and `DefaultCqlType` implementations for a struct.
#[proc_macro_derive(IntoValue, attributes(stargate))]
pub fn derive_into_value(tokens: TokenStream) -> TokenStream {
    let parsed = syn::parse(tokens).unwrap();
    let udt: Udt = Udt::from_derive_input(&parsed).unwrap();
    let udt_type = udt.ident;

    let obj = syn::Ident::new("obj", proc_macro2::Span::mixed_site());
    let fields: Vec<_> = get_fields(udt.data)
        .into_iter()
        .filter(|f| !f.skip)
        .collect();
    let remote_field_names = field_names(&fields);
    let field_values: Vec<_> = convert_to_values(&obj, &fields);

    let result = quote! {
        impl stargate_grpc::into_value::IntoValue<stargate_grpc::types::Udt> for #udt_type {
            fn into_value(self) -> stargate_grpc::Value {
                let #obj = self;
                let mut fields = std::collections::HashMap::new();
                #(fields.insert(#remote_field_names.to_string(), #field_values));*;
                stargate_grpc::Value::raw_udt(fields)
            }
        }
        impl stargate_grpc::into_value::DefaultCqlType for #udt_type {
            type C = stargate_grpc::types::Udt;
        }
    };
    result.into()
}

/// Derives the `IntoValues` impl that allows to use struct in `QueryBuilder::bind`
#[proc_macro_derive(IntoValues, attributes(stargate))]
pub fn derive_into_values(tokens: TokenStream) -> TokenStream {
    let parsed = syn::parse(tokens).unwrap();
    let udt: Udt = Udt::from_derive_input(&parsed).unwrap();
    let udt_type = udt.ident;

    let obj = syn::Ident::new("obj", proc_macro2::Span::mixed_site());
    let fields: Vec<_> = get_fields(udt.data)
        .into_iter()
        .filter(|f| !f.skip)
        .collect();
    let field_names = field_names(&fields);
    let field_values: Vec<_> = convert_to_values(&obj, &fields);

    let result = quote! {
        impl std::convert::From<#udt_type> for stargate_grpc::proto::Values {
            fn from(#obj: #udt_type) -> Self {
                stargate_grpc::proto::Values {
                     value_names: vec![#(#field_names.to_string()),*],
                     values: vec![#(#field_values),*]
                }
            }
        }
    };
    result.into()
}

/// Emits code for reading the field from a hashmap and converting it to proper type.
/// Applies default value if the key is missing in the hashmap or if the value
/// under the key is null.
fn convert_from_hashmap_value(hashmap: &syn::Ident, field: &UdtField) -> TokenStream2 {
    let field_name = field
        .name
        .clone()
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
    let field_type = &field.ty;

    let default_expr = match &field.default {
        None => quote! { Err(ConversionError::field_not_found::<_, Self>(&#hashmap, #field_name)) },
        Some(Override::Inherit) => quote! { Ok(std::default::Default::default()) },
        Some(Override::Explicit(s)) => {
            let path = token_stream(s);
            quote! { Ok(#path) }
        }
    };

    quote! {
        match #hashmap.remove(#field_name) {
            Some(value) => {
                let maybe_value: Option<#field_type> = value.try_into()?;
                match maybe_value {
                    Some(v) => Ok(v),
                    None => #default_expr
                }
            }
            None => #default_expr
        }
    }
}

/// Derives the `TryFromValue` implementation for a struct.
#[proc_macro_derive(TryFromValue, attributes(stargate))]
pub fn derive_try_from_value(tokens: TokenStream) -> TokenStream {
    let parsed = syn::parse(tokens).unwrap();
    let udt: Udt = Udt::from_derive_input(&parsed).unwrap();
    let ident = udt.ident;
    let fields = get_fields(udt.data);
    let field_idents = field_idents(&fields);
    let udt_hashmap = syn::Ident::new("fields", proc_macro2::Span::mixed_site());
    let field_values = fields
        .iter()
        .map(|field| convert_from_hashmap_value(&udt_hashmap, field));

    let result = quote! {

        impl stargate_grpc::from_value::TryFromValue for #ident {
            fn try_from(value: stargate_grpc::Value) ->
                Result<Self, stargate_grpc::error::ConversionError>
            {
                use stargate_grpc::Value;
                use stargate_grpc::error::ConversionError;
                use stargate_grpc::proto::*;
                match value.inner {
                    Some(value::Inner::Udt(UdtValue { mut #udt_hashmap })) => {
                        Ok(#ident {
                            #(#field_idents: #field_values?),*
                        })
                    }
                    other => Err(ConversionError::incompatible::<_, Self>(other))
                }
            }
        }

        impl std::convert::TryFrom<stargate_grpc::Value> for #ident {
            type Error = stargate_grpc::error::ConversionError;
            fn try_from(value: stargate_grpc::Value) ->
                Result<Self, stargate_grpc::error::ConversionError>
            {
                <#ident as stargate_grpc::from_value::TryFromValue>::try_from(value)
            }
        }
    };

    result.into()
}

/// Derives the `TryFromRow` implementation for a struct.
#[proc_macro_derive(TryFromRow, attributes(stargate))]
pub fn derive_try_from_typed_row(tokens: TokenStream) -> TokenStream {
    let parsed = syn::parse(tokens).unwrap();
    let udt: Udt = Udt::from_derive_input(&parsed).unwrap();
    let ident = udt.ident;
    let fields = get_fields(udt.data);
    let field_idents = field_idents(&fields);
    let field_names = field_names(&fields);
    let indexes = 0..field_idents.len();

    let result = quote! {
        impl stargate_grpc::result::ColumnPositions for #ident {
            fn field_to_column_pos(
                column_positions: std::collections::HashMap<String, usize>
            ) -> Result<Vec<usize>, stargate_grpc::result::MapperError>
            {
                use stargate_grpc::result::MapperError;
                let mut result = Vec::new();
                #(
                    result.push(
                        *column_positions
                            .get(#field_names)
                            .ok_or_else(|| MapperError::ColumnNotFound(#field_names))?
                    );
                )*
                Ok(result)
            }
        }

        impl stargate_grpc::result::TryFromRow for #ident {
            fn try_unpack(
                mut row: stargate_grpc::Row,
                column_positions: &[usize]
            ) -> Result<Self, stargate_grpc::error::ConversionError>
            {
                Ok(#ident {
                    #(#field_idents: row.values[column_positions[#indexes]].take().try_into()?),*
                })
            }
        }
    };

    result.into()
}
