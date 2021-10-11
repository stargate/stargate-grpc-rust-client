use proc_macro::TokenStream;

use darling::util::Override;
use darling::{ast, util, FromDeriveInput, FromField};
use quote::quote;

#[derive(Debug, FromField)]
#[darling(attributes(stargate))]
struct UdtField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    default: Option<Override<String>>,
}

#[derive(Debug, FromDeriveInput)]
struct Udt {
    ident: syn::Ident,
    data: ast::Data<util::Ignored, UdtField>,
}

fn get_fields(udt: &ast::Data<util::Ignored, UdtField>) -> &Vec<UdtField> {
    match udt {
        ast::Data::Struct(s) => &s.fields,
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

#[proc_macro_derive(IntoValue, attributes(stargate))]
pub fn derive_into_value(tokens: TokenStream) -> TokenStream {
    let parsed = syn::parse(tokens).unwrap();
    let udt: Udt = Udt::from_derive_input(&parsed).unwrap();
    let ident = udt.ident;

    let fields = get_fields(&udt.data);
    let field_idents = field_idents(fields);
    let field_names = field_names(fields);

    let result = quote! {

        impl stargate_grpc::into_value::IntoValue<stargate_grpc::types::Udt> for #ident {
            fn into_value(self) -> stargate_grpc::Value {
                use stargate_grpc::Value;
                let mut fields = std::collections::HashMap::new();
                #(fields.insert(#field_names.to_string(), Value::from(self.#field_idents)));*;
                Value::raw_udt(fields)
            }
        }

        impl stargate_grpc::into_value::DefaultGrpcType for #ident {
            type C = stargate_grpc::types::Udt;
        }

    };
    result.into()
}

#[proc_macro_derive(TryFromValue, attributes(stargate))]
pub fn derive_try_from_value(tokens: TokenStream) -> TokenStream {
    let parsed = syn::parse(tokens).unwrap();
    let udt: Udt = Udt::from_derive_input(&parsed).unwrap();
    let ident = udt.ident;
    let fields = get_fields(&udt.data);
    let field_idents = field_idents(fields);
    let field_names = field_names(fields);

    let check_has_default = fields.iter().map(|f| {
        if f.default.is_none() {
            let field_name = f.ident.as_ref().unwrap().to_string();
            quote! {
                if !fields.contains_key(#field_name) {
                    return Err(ConversionError::field_not_found::<_, Self>(
                        fields,
                        #field_name
                    ));
                }
            }
        } else {
            quote! {}
        }
    });

    let field_defaults = fields.iter().map(|f| match &f.default {
        None => quote! { panic!("No default") },
        Some(Override::Inherit) => quote! { Ok(std::default::Default::default()) },
        Some(Override::Explicit(s)) => {
            let path = token_stream(s);
            quote! { Ok(#path()) }
        }
    });

    let result = quote! {

        impl stargate_grpc::from_value::TryFromValue for #ident {
            fn try_from(value: stargate_grpc::Value) ->
                Result<Self, stargate_grpc::error::ConversionError>
            {
                use stargate_grpc::Value;
                use stargate_grpc::error::ConversionError;
                use stargate_grpc::proto::*;
                match value.inner {
                    Some(value::Inner::Udt(UdtValue { mut fields })) => {
                        #(#check_has_default)*
                        Ok(#ident {
                            #(#field_idents: fields
                                .remove(#field_names)
                                .map(|value| value.try_into())
                                .unwrap_or_else(|| #field_defaults)?
                            ),*
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
