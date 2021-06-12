use crate::DECL_FN_NAME;
use quote::quote;
use regex::Regex;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream, Result};
use syn::token;
use syn::{braced, Error, Generics, Ident, Token, Type, Visibility, WhereClause};

use crate::parse_common::parse_inner_attributes;
use crate::parse_gen_array::{parse_gen_arrays, GenArray};
use crate::parse_in_array::{parse_in_array_fields, InArrayElement, InArrayField};

pub struct DeriveArraygen {
    pub gen_arrays: HashMap<Ident, GenArray>,
    pub vis: Visibility,
    pub struct_name: Ident,
    pub generics: Generics,
}

impl Parse for DeriveArraygen {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut gen_arrays = input.call(parse_gen_arrays)?;
        let vis = input.parse::<Visibility>()?;

        let lookahead = input.lookahead1();
        if !lookahead.peek(Token![struct]) {
            return Err(input.error("derive 'Arraygen' should only be used with braced structs"));
        }

        let _ = input.parse::<Token![struct]>()?;
        let struct_name = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>()?;
        let where_clause = parse_struct(input, &mut gen_arrays)?;

        Ok(DeriveArraygen {
            gen_arrays,
            vis,
            struct_name,
            generics: Generics {
                where_clause,
                ..generics
            },
        })
    }
}

pub fn parse_struct(
    input: ParseStream,
    gen_arrays: &mut HashMap<Ident, GenArray>,
) -> Result<Option<WhereClause>> {
    let mut lookahead = input.lookahead1();
    let mut where_clause = None;
    if lookahead.peek(Token![where]) {
        where_clause = Some(input.parse()?);
        lookahead = input.lookahead1();
    }

    if lookahead.peek(token::Brace) {
        parse_braced_struct(input, gen_arrays)?;
        Ok(where_clause)
    } else {
        Err(input.error("derive 'Arraygen' should only be used with braced structs"))
    }
}

pub(crate) fn parse_braced_struct(
    input: ParseStream,
    gen_arrays: &mut HashMap<Ident, GenArray>,
) -> Result<()> {
    let content;
    let _ = braced!(content in input);
    parse_inner_attributes(&content)?;
    let mut err = None;
    content
        .parse_terminated::<InArrayField, Token![,]>(parse_in_array_fields)?
        .into_iter()
        .for_each(|iaf| {
            for (_, ga) in gen_arrays.iter_mut() {
                for implicit_ty in ga.implicit_select_all.iter() {
                    if equal_types(&iaf.ty, implicit_ty) {
                        ga.fields.push(InArrayElement {
                            ident: iaf.ident.clone(),
                            ty: iaf.ty.clone(),
                            cast: None,
                        });
                    }
                }
            }
            for attr in iaf.attrs.iter() {
                for entry in attr.entries.iter() {
                    if let Some(ga) = gen_arrays.get_mut(&entry.ident) {
                        if ga.fields.iter().any(|iae| iae.ident == iaf.ident) {
                            err = Some(Error::new_spanned(
                                entry.ident.clone(),
                                format!(
                                    "Field '{}' is already included in {} method '{}'",
                                    iaf.ident.to_string(),
                                    DECL_FN_NAME,
                                    entry.ident.to_string()
                                ),
                            ));
                        } else {
                            ga.fields.push(InArrayElement {
                                ident: iaf.ident.clone(),
                                ty: iaf.ty.clone(),
                                cast: entry.decorator.clone(),
                            });
                        }
                    } else {
                        err = Some(Error::new_spanned(
                            entry.ident.clone(),
                            format!(
                                "{} method '{}' not present but used by field '{}'",
                                DECL_FN_NAME,
                                entry.ident.to_string(),
                                iaf.ident.to_string()
                            ),
                        ));
                    }
                }
            }
        });
    if let Some(err) = err {
        return Err(err);
    }
    Ok(())
}

fn equal_types(field_ty: &Type, implicit_ty: &Type) -> bool {
    if *field_ty == *implicit_ty {
        return true;
    }
    if let Type::Infer(_) = implicit_ty {
        return true;
    }

    let mut implicit_iter = quote! { #implicit_ty }.into_iter();

    if !implicit_iter.any(|t| matches!(t, proc_macro2::TokenTree::Ident(p) if p == "_")) {
        return false;
    }

    let implicit_string = implicit_iter
        .map(|t| t.to_string())
        .map(|s| if s == "_" { ".+".to_string() } else { s })
        .collect::<Vec<String>>()
        .join("");

    let field_string = quote! { #field_ty }
        .into_iter()
        .map(|t| t.to_string())
        .collect::<Vec<String>>()
        .join("");

    match Regex::new(implicit_string.as_ref()) {
        Ok(re) => re.is_match(field_string.as_ref()),
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn equal_types_exactly_same_returns_true() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<i32>").unwrap(),
                &syn::parse_str("Option<i32>").unwrap()
            ),
            true
        );
    }

    #[test]
    fn equal_types_compared_to_wildcard_returns_true() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<i32>").unwrap(),
                &syn::parse_str("_").unwrap()
            ),
            true
        );
    }

    #[test]
    fn equal_types_compared_to_option_wildcard_returns_true() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<i32>").unwrap(),
                &syn::parse_str("Option<_>").unwrap()
            ),
            true
        );
    }

    #[test]
    fn equal_types_exactly_same_returns_true2() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<Result<(i32, f32), std::fmt::Error>>").unwrap(),
                &syn::parse_str("Option<Result<(i32, f32), std::fmt::Error>>").unwrap()
            ),
            true
        );
    }

    #[test]
    fn equal_types_different1_returns_false() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<Result<(i32, f32), std::fmt::Error>>").unwrap(),
                &syn::parse_str("Option<Result<(i32, i32), std::fmt::Error>>").unwrap()
            ),
            false
        );
    }

    #[test]
    fn equal_types_different2_returns_false() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<Result<(i32, f32), std::error::Error>>").unwrap(),
                &syn::parse_str("Option<Result<(i32, f32), std::fmt::Error>>").unwrap()
            ),
            false
        );
    }

    #[test]
    fn equal_types_compared_to_wildcard_returns_true2() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<i32>").unwrap(),
                &syn::parse_str("_").unwrap()
            ),
            true
        );
    }

    #[test]
    fn equal_types_compared_to_wildcard2_returns_true() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<i32>").unwrap(),
                &syn::parse_str("Option<_>").unwrap()
            ),
            true
        );
    }

    #[test]
    fn equal_types_compared_to_wildcard3_returns_true() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<Option<i32>>").unwrap(),
                &syn::parse_str("Option<Option<_>>").unwrap()
            ),
            true
        );
    }

    #[test]
    fn equal_types_compared_to_wildcard4_returns_true() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<Result<i32, i32>>").unwrap(),
                &syn::parse_str("Option<Result<_, i32>>").unwrap()
            ),
            true
        );
    }

    #[test]
    fn equal_types_compared_to_wildcard5_returns_true() {
        assert_eq!(
            equal_types(
                &syn::parse_str("Option<Result<i32, i32>>").unwrap(),
                &syn::parse_str("Option<Result<_, f32>>").unwrap()
            ),
            false
        );
    }
}
