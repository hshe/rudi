use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, Ident, ItemStruct};

use crate::{
    provider_attribute::{ProviderAttribute, SimpleAttribute},
    utils::{Color, Scope},
};

pub(crate) fn generate(
    attribute: ProviderAttribute,
    mut item_struct: ItemStruct,
    scope: Scope,
) -> syn::Result<TokenStream> {
    let SimpleAttribute {
        name,
        eager_create,
        binds,
        async_constructor,
        not_auto_register,
    } = attribute.simplify();

    #[cfg(feature = "auto-register")]
    crate::utils::check_auto_register_with_generics(
        not_auto_register,
        &item_struct.generics,
        "struct",
        scope,
    )?;

    let color = if async_constructor {
        Color::Async
    } else {
        Color::Sync
    };

    let fields_attrs = get_attrs_from_fields(&mut item_struct.fields, color)?;

    let create_provider = crate::utils::get_create_provider(scope, color);

    let struct_ident = &item_struct.ident;

    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    let instance = match fields_attrs {
        FieldsAttributes::Unit => quote! {
            #struct_ident
        },
        FieldsAttributes::Named(idents, resolve_methods) => {
            quote! {
                #struct_ident {
                    #(
                        #idents: #resolve_methods,
                    )*
                }
            }
        }
        FieldsAttributes::Unnamed(resolve_methods) => {
            quote! {
                #struct_ident(
                    #(
                        #resolve_methods,
                    )*
                )
            }
        }
    };

    let constructor = match color {
        Color::Async => {
            quote! {
                #[allow(unused_variables)]
                |cx| ::std::boxed::Box::pin(async {
                    #instance
                })
            }
        }
        Color::Sync => {
            quote! {
                #[allow(unused_variables)]
                |cx| #instance
            }
        }
    };

    let auto_register = if not_auto_register {
        quote! {}
    } else {
        #[cfg(feature = "auto-register")]
        quote! {
            ::rudi::register_provider!(<#struct_ident as ::rudi::DefaultProvider>::provider());
        }
        #[cfg(not(feature = "auto-register"))]
        quote! {}
    };

    let expand = quote! {
        #item_struct

        impl #impl_generics ::rudi::DefaultProvider for #struct_ident #ty_generics #where_clause {
            type Type = Self;

            fn provider() -> ::rudi::Provider<Self> {
                <::rudi::Provider<_> as ::core::convert::From<_>>::from(
                    ::rudi::#create_provider(#constructor)
                        .name(#name)
                        .eager_create(#eager_create)
                        #binds
                )
            }
        }

        #auto_register
    };

    Ok(expand)
}

enum FieldsAttributes {
    Unit,
    Named(Vec<Ident>, Vec<TokenStream>),
    Unnamed(Vec<TokenStream>),
}

fn get_attrs_from_fields(fields: &mut Fields, color: Color) -> syn::Result<FieldsAttributes> {
    match fields {
        Fields::Unit => Ok(FieldsAttributes::Unit),
        Fields::Named(FieldsNamed { named, .. }) => {
            let len = named.len();
            let mut idents = Vec::with_capacity(len);
            let mut resolve_methods = Vec::with_capacity(len);

            for Field { attrs, ident, .. } in named {
                resolve_methods.push(crate::utils::get_one_arg_or_field_resolve_expr(
                    attrs, color,
                )?);
                idents.push(ident.clone().unwrap());
            }

            Ok(FieldsAttributes::Named(idents, resolve_methods))
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let mut resolve_methods = Vec::with_capacity(unnamed.len());

            for Field { attrs, .. } in unnamed {
                resolve_methods.push(crate::utils::get_one_arg_or_field_resolve_expr(
                    attrs, color,
                )?);
            }

            Ok(FieldsAttributes::Unnamed(resolve_methods))
        }
    }
}
