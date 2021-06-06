use crate::DECL_FN_NAME;
use std::collections::HashMap;
use syn::{Error, Ident, Token, Visibility, Type, braced, WhereClause, Generics};
use syn::parse::{Parse, ParseStream, Result};
use syn::token;
use quote::quote;
use crate::parse_gen_array::{GenArray, parse_gen_arrays};
use crate::parse_in_array::{InArrayField, InArrayElement, parse_in_array_fields};
use crate::parse_common::parse_inner_attributes;

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
            return Err(lookahead.error());
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

pub fn parse_struct(input: ParseStream, gen_arrays: &mut HashMap<Ident, GenArray>) -> Result<Option<WhereClause>> {
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
        Err(lookahead.error())
    }
}

pub(crate) fn parse_braced_struct(input: ParseStream, gen_arrays: &mut HashMap<Ident, GenArray>) -> Result<()> {
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
                            cast: None
                        });
                    }
                } 
            } 
            for attr in iaf.attrs.iter() {
                for entry in attr.entries.iter() {
                    if let Some(ga) = gen_arrays.get_mut(&entry.ident) {
                        if ga.fields.iter().any(|iae| iae.ident == iaf.ident) {
                            err = Some(Error::new_spanned(entry.ident.clone(), format!("Field '{}' is already included in {} method '{}'", iaf.ident.to_string(), DECL_FN_NAME, entry.ident.to_string())));
                        } else {
                            ga.fields.push(InArrayElement {
                                ident: iaf.ident.clone(),
                                ty: iaf.ty.clone(),
                                cast: entry.decorator.clone()
                            });
                        }
                    } else {
                        err = Some(Error::new_spanned(entry.ident.clone(), format!("{} method '{}' not present but used by field '{}'", DECL_FN_NAME, entry.ident.to_string(), iaf.ident.to_string())));
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

    let field_ts = quote! { #field_ty };
    let implicit_ts = quote! { #implicit_ty };

    let mut field_iter = field_ts.into_iter();
    let mut next_field = field_iter.next();
    let mut can_be_wildcard = true;
    let mut expecting_closing = false;
    for implicit_token in implicit_ts.into_iter() {
        if expecting_closing {
            if implicit_token.to_string() == ">" {
                expecting_closing = false;
                next_field = field_iter.next();
                continue;
            }
        } else if let Some(ref next) = next_field {
            if implicit_token.to_string() == next.to_string() {
                next_field = field_iter.next();
                can_be_wildcard = implicit_token.to_string() == "<";
                continue;
            }
            if can_be_wildcard && implicit_token.to_string() == "_" {
                can_be_wildcard = false;
                expecting_closing = true;
                while let Some(ref next) = next_field {
                    if next.to_string() == ">" {
                        break;
                    }
                    next_field = field_iter.next();
                }
                continue;
            }
        }
        return false;
    }

    true
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn equal_types_exactly_same_returns_true() {
        assert_eq!(equal_types(&syn::parse_str("Option<i32>").unwrap(), &syn::parse_str("Option<i32>").unwrap()), true);
    }

    #[test]
    fn equal_types_compared_to_wildcard_returns_true() {
        assert_eq!(equal_types(&syn::parse_str("Option<i32>").unwrap(), &syn::parse_str("_").unwrap()), true);
    }

    #[test]
    fn equal_types_compared_to_option_wildcard_returns_true() {
        assert_eq!(equal_types(&syn::parse_str("Option<i32>").unwrap(), &syn::parse_str("Option<_>").unwrap()), true);
    }
}




















