use crate::DECL_FN_NAME;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream, Result};
use syn::token;
use syn::{braced, Error, Generics, Ident, Token, Visibility, WhereClause};

use crate::parse_attribute::parse_inner_attributes;
use crate::parse_gen_array::{parse_gen_arrays, GenArray};
use crate::parse_in_array::{
    parse_in_array_fields, InArrayElement, InArrayElementKind, InArrayField,
};
use crate::types::ty_inferred_by;

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
    for iaf in content
        .parse_terminated::<InArrayField, Token![,]>(parse_in_array_fields)?
        .into_iter()
    {
        for (_, ga) in gen_arrays.iter_mut() {
            for implicit_ty in ga.implicit_select_all_tys.iter() {
                if ty_inferred_by(&iaf.ty, implicit_ty) {
                    ga.fields.push(InArrayElement {
                        ident: iaf.ident.clone(),
                        ty: iaf.ty.clone(),
                        cast: ga.implicit_select_all_decorator.cast.clone(),
                        kind: InArrayElementKind::Implicit,
                    });
                }
            }
        }
        for attr in iaf.attrs.iter() {
            for entry in attr.entries.iter() {
                if let Some(ga) = gen_arrays.get_mut(&entry.ident) {
                    let iae = ga.fields.iter().find(|iae| iae.ident == iaf.ident);
                    if matches!(iae, Some(iae) if iae.kind != InArrayElementKind::Implicit || (iae.kind == InArrayElementKind::Implicit && !entry.decorator.override_implicit))
                    {
                        return Err(Error::new_spanned(
                            entry.ident.clone(),
                            format!(
                                "Field '{}' is already included in {} method '{}'",
                                iaf.ident.to_string(),
                                DECL_FN_NAME,
                                entry.ident.to_string()
                            ),
                        ));
                    } else {
                        if let Some(ident) = iae.map(|iae| iae.ident.clone()) {
                            ga.fields.retain(|field| field.ident != ident);
                        }

                        ga.fields.push(InArrayElement {
                            ident: iaf.ident.clone(),
                            ty: iaf.ty.clone(),
                            cast: entry.decorator.cast.clone(),
                            kind: InArrayElementKind::InArray,
                        });
                    }
                } else {
                    return Err(Error::new_spanned(
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
    }
    Ok(())
}
