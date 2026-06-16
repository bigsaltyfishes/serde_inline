//! A proc macro for inlining serde deserialization logic.
//!
//! This crate provides a `#[serde_inline]` attribute that allows you to inline
//! serde deserialization logic directly into your struct fields.
//!
//! # Example
//! ```ignore
//! use serde::Deserialize;
//! use serde_inline::serde_inline;
//!
//! #[derive(Deserialize)]
//! struct Example {
//!     #[serde_inline(
//!         deserialize_with = |deserializer| {
//!             RawVec3::deserialize(deserializer).map(Into::into)
//!         },
//!         default = Vec3(RawVec3 { x: 0, y: 0, z: 0 })
//!     )]
//!     position: Vec3,
//! }
//! ```
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Expr, Fields, Item, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn serde_inline(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item);
    let mut helpers = Vec::new();
    let mut targets = Vec::new();

    match &mut input {
        Item::Struct(item_struct) => {
            targets.push((item_struct.ident.clone(), &mut item_struct.fields));
        }
        Item::Enum(item_enum) => {
            let enum_name = &item_enum.ident;
            for variant in &mut item_enum.variants {
                let variant_name = &variant.ident;
                let combined_ident = format_ident!("{enum_name}_{variant_name}");
                targets.push((combined_ident, &mut variant.fields));
            }
        }
        _ => {
            return syn::Error::new_spanned(input, "serde_inline only supports structs and enums")
                .to_compile_error()
                .into();
        }
    }

    for (parent_ident, fields) in targets {
        let field_iter: Box<dyn Iterator<Item = (String, &mut syn::Field)>> = match fields {
            Fields::Named(fields_named) => Box::new(
                fields_named
                    .named
                    .iter_mut()
                    .map(|f| (f.ident.as_ref().unwrap().to_string(), f)),
            ),
            Fields::Unnamed(fields_unnamed) => Box::new(
                fields_unnamed
                    .unnamed
                    .iter_mut()
                    .enumerate()
                    .map(|(i, f)| (format!("_{i}"), f)),
            ),
            Fields::Unit => continue,
        };

        for (field_name, field) in field_iter {
            let mut inline_attr_index = None;
            let mut deserialize_expr: Option<Expr> = None;
            let mut default_expr: Option<Expr> = None;

            for (i, attr) in field.attrs.iter().enumerate() {
                if attr.path().is_ident("serde_inline") {
                    inline_attr_index = Some(i);

                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("deserialize_with") {
                            deserialize_expr = Some(meta.value()?.parse()?);
                            Ok(())
                        } else if meta.path.is_ident("default") {
                            default_expr = Some(meta.value()?.parse()?);
                            Ok(())
                        } else {
                            Err(meta.error("unsupported serde_inline property"))
                        }
                    })
                    .unwrap();

                    break;
                }
            }

            let Some(idx) = inline_attr_index else {
                continue;
            };
            field.attrs.remove(idx);

            let field_type = &field.ty;
            let mut serde_args = Vec::new();

            if let Some(expr) = deserialize_expr {
                let function_name = format!("_{parent_ident}_{field_name}_deserialize");
                let function_ident = format_ident!("{function_name}");
                serde_args.push(quote! { deserialize_with = #function_name });
                helpers.push(quote! {
                    fn #function_ident<'de, D>(deserializer: D) -> ::std::result::Result<#field_type, D::Error>
                    where
                        D: ::serde::Deserializer<'de>,
                    {
                        let parser = #expr;
                        parser(deserializer)
                    }
                });
            }

            if let Some(expr) = default_expr {
                let function_name = format!("_{parent_ident}_{field_name}_default");
                let function_ident = format_ident!("{function_name}");
                serde_args.push(quote! { default = #function_name });
                helpers.push(quote! {
                    fn #function_ident() -> #field_type {
                        #expr
                    }
                });
            }

            if !serde_args.is_empty() {
                let attr: Attribute = parse_quote! { #[serde( #(#serde_args),* )] };
                field.attrs.push(attr);
            }
        }
    }

    let expanded = quote! {
        #input
        #(#helpers)*
    };
    TokenStream::from(expanded)
}
