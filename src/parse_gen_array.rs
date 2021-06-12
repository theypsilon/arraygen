use quote::quote;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream, Result};
use syn::token;
use syn::{bracketed, parenthesized, Error, Ident, Path, Token, Type, Visibility};

use crate::parse_attribute::single_parse_outer_attribute;
use crate::parse_decorator::{CastKind, Decorator};
use crate::parse_in_array::InArrayElement;
use crate::types::are_matching_types;
use crate::{DECL_FN_NAME, IMPLICIT_SELECT_ALL_NAME};

pub struct GenArray {
    pub vis: Visibility,
    pub fn_name: Ident,
    pub fn_ty: Type,
    pub is_mut: bool,
    pub is_ref: bool,
    pub implicit_select_all_tys: Vec<Type>,
    pub implicit_select_all_decorator: Decorator,
    pub casts: Vec<(syn::Ident, syn::Type, proc_macro2::TokenStream, CastKind)>,
    pub fields: Vec<InArrayElement>,
}

pub fn parse_gen_arrays(input: ParseStream) -> Result<HashMap<Ident, GenArray>> {
    let mut gen_arrays = HashMap::new();
    while input.peek(Token![#]) {
        if let Ok(true) = is_gen_array(&input.fork()) {
            let gen_array = input.call(single_parse_gen_array)?;
            gen_arrays.insert(gen_array.fn_name.clone(), gen_array);
        } else {
            input.call(single_parse_outer_attribute)?;
        }
    }
    Ok(gen_arrays)
}

pub fn single_parse_gen_array(input: ParseStream) -> Result<GenArray> {
    let content;
    let _: Token![#] = input.parse()?;
    let _: token::Bracket = bracketed!(content in input);
    let path: Path = content.call(Path::parse_mod_style)?;

    if path.segments.len() != 1 {
        return Err(input.error(format!("Wrong syntax for {}", DECL_FN_NAME)));
    }

    content.call(parse_gen_array_group)
}

pub fn is_gen_array(input: ParseStream) -> Result<bool> {
    let content;
    let _: Token![#] = input.parse()?;
    let _: token::Bracket = bracketed!(content in input);
    let path: Path = content.call(Path::parse_mod_style)?;

    Ok(!path.segments.is_empty() && path.segments[0].ident == DECL_FN_NAME)
}

pub fn parse_gen_array_group(input: ParseStream) -> Result<GenArray> {
    let content;
    let _ = parenthesized!(content in input);
    let vis: Visibility = content.parse()?;
    let _: Token![fn] = content.parse()?;
    let fn_name: Ident = content.parse()?;
    let _: Token![:] = content.parse()?;
    let fn_ty: Type = content.parse()?;

    let (is_ref, is_mut) = if let Type::Reference(ref reference) = fn_ty {
        (true, reference.mutability.is_some())
    } else {
        (false, false)
    };

    let mut implicit_select_all_tys = vec![];
    let mut implicit_select_all_decorator = Decorator::new();

    if content.peek(Token![,]) && content.peek2(syn::Ident) {
        let _: Token![,] = content.parse()?;
        let implicit: syn::Ident = content.parse()?;
        if implicit != IMPLICIT_SELECT_ALL_NAME {
            return Err(content.error(format!("clause '{}' not recognised", implicit)));
        }

        implicit_select_all_decorator = content.parse::<Decorator>()?;
        if implicit_select_all_decorator.override_implicit {
            return Err(Error::new_spanned(
                implicit,
                format!(
                    "{} method '{}' contains {} clause with forbidden decorator 'override_implicit'",
                    DECL_FN_NAME,
                    fn_name,
                    IMPLICIT_SELECT_ALL_NAME
                ),
            ));
        }

        let _: Token![:] = content.parse::<Token![:]>()?;
        implicit_select_all_tys = content
            .parse_terminated::<Type, Token![,]>(Type::parse)?
            .into_iter()
            .collect::<Vec<Type>>();

        if implicit_select_all_tys.is_empty() {
            return Err(content.error("missing type to select"));
        }

        for (i, ty_left) in implicit_select_all_tys.iter().enumerate() {
            for ty_right in implicit_select_all_tys.iter().skip(i + 1) {
                if are_matching_types(ty_left, ty_right) {
                    return Err(Error::new_spanned(
                        ty_right,
                        format!(
                            "{} method '{}' contains {} clause with duplicated '{}' type",
                            DECL_FN_NAME,
                            fn_name,
                            IMPLICIT_SELECT_ALL_NAME,
                            quote! { #ty_right }.to_string()
                        ),
                    ));
                }
            }
        }
    }

    Ok(GenArray {
        vis,
        fn_name,
        fn_ty,
        is_mut,
        is_ref,
        implicit_select_all_tys,
        implicit_select_all_decorator,
        casts: vec![],
        fields: vec![],
    })
}
