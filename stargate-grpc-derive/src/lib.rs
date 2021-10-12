use proc_macro::TokenStream;

use darling::util::Override;
use darling::{ast, util, FromDeriveInput, FromField};
use quote::quote;
use syn::__private::TokenStream2;

#[derive(Debug, FromField)]
#[darling(attributes(stargate))]
struct UdtField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    default: Option<Override<String>>,
    #[darling(default)]
    grpc_type: Option<String>,
    #[darling(default)]
    skip: bool,
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

fn field_names(fields: &[UdtField]) -> Vec<String> {
    fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect()
}

fn token_stream(s: &str) -> proc_macro2::TokenStream {
    s.parse().unwrap()
}

/// Emits code for reading the field value and converting it to proper `Value` object.
fn convert_to_value(struc: &syn::Ident, field: &UdtField) -> TokenStream2 {
    let field_ident = field.ident.as_ref().unwrap();
    match &field.grpc_type {
        Some(t) => {
            let grpc_type = token_stream(t.as_str());
            quote! { Value::of_type(#grpc_type, #struc.#field_ident) }
        }
        None => {
            quote! { Value::from(#struc.#field_ident) }
        }
    }
}

#[proc_macro_derive(IntoValue, attributes(stargate))]
pub fn derive_into_value(tokens: TokenStream) -> TokenStream {
    let parsed = syn::parse(tokens).unwrap();
    let udt: Udt = Udt::from_derive_input(&parsed).unwrap();
    let udt_type = udt.ident;

    let struct_var = syn::Ident::new("udt", proc_macro2::Span::mixed_site());
    let fields: Vec<_> = get_fields(udt.data)
        .into_iter()
        .filter(|f| !f.skip)
        .collect();
    let field_names = field_names(&fields);
    let field_values: Vec<_> = fields
        .iter()
        .map(|f| convert_to_value(&struct_var, f))
        .collect();

    let result = quote! {

        impl stargate_grpc::into_value::IntoValue<stargate_grpc::types::Udt> for #udt_type {
            fn into_value(self) -> stargate_grpc::Value {
                use stargate_grpc::Value;
                let #struct_var = self;
                let mut fields = std::collections::HashMap::new();
                #(fields.insert(#field_names.to_string(), #field_values));*;
                Value::raw_udt(fields)
            }
        }

        impl stargate_grpc::into_value::DefaultGrpcType for #udt_type {
            type C = stargate_grpc::types::Udt;
        }

        impl std::convert::From<#udt_type> for stargate_grpc::proto::Values {
            fn from(#struct_var: #udt_type) -> Self {
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
    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_type = &field.ty;

    let default_expr = match &field.default {
        None => quote! { Err(ConversionError::field_not_found::<_, Self>(&#hashmap, #field_name)) },
        Some(Override::Inherit) => quote! { Ok(std::default::Default::default()) },
        Some(Override::Explicit(s)) => {
            let path = token_stream(s);
            quote! { Ok(#path()) }
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
