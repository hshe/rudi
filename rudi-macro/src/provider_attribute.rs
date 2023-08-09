use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Expr, ExprPath, Meta, MetaNameValue, Path, Token,
};

use crate::name::Name;

pub(crate) struct ProviderAttribute {
    name: Option<(Path, Name)>,
    eager_create: Option<Path>,
    binds: Option<(Path, Vec<ExprPath>)>,
    pub(crate) async_constructor: Option<Path>,
    not_auto_register: Option<Path>,
}

impl Parse for ProviderAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name: Option<(Path, Name)> = None;
        let mut eager_create: Option<Path> = None;
        let mut binds: Option<(Path, Vec<ExprPath>)> = None;
        let mut async_constructor: Option<Path> = None;
        let mut not_auto_register: Option<Path> = None;

        let attr = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in attr {
            let meta_path = meta.path();
            let meta_path_span = meta_path.span();

            macro_rules! check_duplicate {
                ($attribute:tt) => {
                    if $attribute.is_some() {
                        return Err(syn::Error::new(
                            meta_path_span,
                            concat!(
                                "the `",
                                stringify!($attribute),
                                "` attribute can only be set once"
                            ),
                        ));
                    }
                };
            }

            if meta_path.is_ident("name") {
                check_duplicate!(name);

                let MetaNameValue { path, value, .. } = require_name_value(meta)?;

                let _name = Name::try_from((
                    value,
                    "the value of `name` must be a literal string or an expression call",
                ))?;

                name = Some((path, _name));
                continue;
            }

            if meta_path.is_ident("eager_create") {
                check_duplicate!(eager_create);

                eager_create = Some(require_path_only(meta)?);
                continue;
            }

            if meta_path.is_ident("binds") {
                check_duplicate!(binds);

                let MetaNameValue { path, value, .. } = require_name_value(meta)?;

                let array = if let Expr::Array(array) = value {
                    array
                } else {
                    return Err(syn::Error::new(
                        value.span(),
                        "the value of `binds` must be an array",
                    ));
                };

                let mut paths = vec![];

                for expr in array.elems {
                    if let Expr::Path(path) = expr {
                        paths.push(path);
                    } else {
                        return Err(syn::Error::new(
                            expr.span(),
                            "the element in `binds` must be an expression path",
                        ));
                    }
                }

                binds = Some((path, paths));
                continue;
            }

            if meta_path.is_ident("async_constructor") {
                check_duplicate!(async_constructor);

                async_constructor = Some(require_path_only(meta)?);
                continue;
            }

            if meta_path.is_ident("not_auto_register") {
                check_duplicate!(not_auto_register);

                not_auto_register = Some(require_path_only(meta)?);
                continue;
            }

            return Err(syn::Error::new(
                meta_path_span,
                 "the attribute must be one of: `name`, `eager_create`, `binds`, `async_constructor`, `not_auto_register`",

            ));
        }

        Ok(ProviderAttribute {
            name,
            eager_create,
            binds,
            async_constructor,
            not_auto_register,
        })
    }
}

impl ProviderAttribute {
    pub(crate) fn simplify(&self) -> SimpleAttribute {
        let ProviderAttribute {
            name,
            eager_create,
            binds,
            async_constructor,
            not_auto_register,
        } = self;

        SimpleAttribute {
            name: match name {
                Some((_, name)) => quote! {
                    #name
                },
                None => quote! {
                    ""
                },
            },
            eager_create: eager_create.is_some(),
            binds: if let Some((_, binds)) = binds {
                quote! {
                    #(
                        .bind(#binds)
                    )*
                }
            } else {
                quote! {}
            },
            async_constructor: async_constructor.is_some(),
            not_auto_register: not_auto_register.is_some(),
        }
    }
}

pub(crate) struct SimpleAttribute {
    pub(crate) name: TokenStream,
    pub(crate) eager_create: bool,
    pub(crate) binds: TokenStream,
    pub(crate) async_constructor: bool,
    pub(crate) not_auto_register: bool,
}

fn require_path_only(meta: Meta) -> syn::Result<Path> {
    meta.require_path_only()?;

    match meta {
        Meta::Path(path) => Ok(path),
        _ => unreachable!(),
    }
}

fn require_name_value(meta: Meta) -> syn::Result<MetaNameValue> {
    meta.require_name_value()?;

    match meta {
        Meta::NameValue(name_value) => Ok(name_value),
        _ => unreachable!(),
    }
}
