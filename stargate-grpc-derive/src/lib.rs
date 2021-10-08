use proc_macro::TokenStream;

use darling::{ast, util, FromDeriveInput, FromField};
use quote::quote;

#[derive(Debug, FromField)]
struct UdtField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(default)]
    skip: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
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

fn field_idents(fields: &Vec<UdtField>) -> Vec<&syn::Ident> {
    fields.iter().map(|f| f.ident.as_ref().unwrap()).collect()
}

fn field_names(fields: &Vec<UdtField>) -> Vec<String> {
    fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect()
}

#[proc_macro_derive(IntoValue)]
pub fn derive_udt_into_value(tokens: TokenStream) -> TokenStream {
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

#[proc_macro_derive(TryFromValue)]
pub fn derive_udt_try_from_value(tokens: TokenStream) -> TokenStream {
    let parsed = syn::parse(tokens).unwrap();
    let udt: Udt = Udt::from_derive_input(&parsed).unwrap();
    let ident = udt.ident;
    let fields = get_fields(&udt.data);
    let field_idents = field_idents(fields);
    let field_names = field_names(fields);

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
                        #(if !fields.contains_key(#field_names) {
                            return Err(ConversionError::field_not_found::<_, Self>(
                                fields,
                                #field_names
                            ));
                        })*
                        Ok(#ident {
                            #(#field_idents: fields
                                .remove(#field_names)
                                .unwrap()
                                .try_into()?
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
                value.try_into()
            }
        }
    };
    result.into()
}
