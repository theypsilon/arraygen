use crate::casting::CastKind;
use crate::FIELD_SELECTOR_NAME;
use crate::parse_common::single_parse_outer_attribute;
use syn::{Error, Ident, Token, Visibility, Type, braced, parenthesized, bracketed, Path};
use syn::parse::{ParseStream, Result};
use syn::token;

pub struct InArrayElement {
    pub ident: Ident,
    pub ty: Type,
    pub cast: Option<CastKind>
}

pub struct InArrayAttributeEntry {
    pub ident: Ident,
    pub decorator: Option<CastKind>
}

pub struct InArrayAttribute {
    pub entries: Vec<InArrayAttributeEntry>
}

pub struct InArrayField {
    pub attrs: Vec<InArrayAttribute>,
    pub ident: Ident,
    pub ty: Type
}

pub fn parse_in_array_fields(input: ParseStream) -> Result<InArrayField> {
    let attrs: Vec<InArrayAttribute> = input.call(parse_in_array_attributes)?;
    let _: Visibility = input.parse()?;
    let ident: Ident = input.parse()?;
    let _: Option<token::Colon> = Some(input.parse()?);
    let ty: Type = input.parse()?;
    Ok(InArrayField {
        attrs,
        ident,
        ty
    })
}

pub fn is_in_array_attribute(input: ParseStream) -> Result<bool> {
    let content;
    let _: Token![#] = input.parse()?;
    let _: token::Bracket = bracketed!(content in input);
    let path: Path = content.call(Path::parse_mod_style)?;

    Ok(!path.segments.is_empty() && path.segments[0].ident == FIELD_SELECTOR_NAME)
}

pub fn parse_in_array_attributes(input: ParseStream) -> Result<Vec<InArrayAttribute>> {
    let mut ret = vec![];
    while input.peek(Token![#]) {
        if let Ok(true) = is_in_array_attribute(&input.fork()) {
            ret.push(input.call(parse_single_in_array_attribute_header)?);
        } else {
            input.call(single_parse_outer_attribute)?;
        }
    }
    Ok(ret)
}


pub fn parse_single_in_array_attribute_header(input: ParseStream) -> Result<InArrayAttribute> {
    let content;
    let _: Token![#] = input.parse()?;
    let _: token::Bracket = bracketed!(content in input);
    let path: Path = content.call(Path::parse_mod_style)?;

    if path.segments.len() != 1 {
        return Err(input.error(format!("Wrong syntax for {}", FIELD_SELECTOR_NAME)));
    }

    content.call(parse_single_in_array_attribute_body)
}

pub fn parse_single_in_array_attribute_body(input: ParseStream) -> Result<InArrayAttribute> {
    let content;
    let _ = parenthesized!(content in input);
    Ok(InArrayAttribute {
        entries: content
            .parse_terminated::<InArrayAttributeEntry, Token![,]>(parse_attribute_entry)?
            .into_iter()
            .collect()
    })
}

pub fn parse_attribute_entry(input: ParseStream) -> Result<InArrayAttributeEntry> {
    let ident: Ident = input.parse()?;
    if input.peek(token::Brace) {
        let content;
        let _ = braced!(content in input);
        let deco_ident = content.parse::<Ident>()?;
        let decorator = match deco_ident.to_string().as_ref() {
            "cast" => CastKind::SafeCast,
            "unsafe_transmute" => CastKind::UnsafeTransmute,
            decorator => return Err(Error::new_spanned(deco_ident, format!("{} doesn't allow '{}' as decorator", FIELD_SELECTOR_NAME, decorator)))
        };
        return Ok(InArrayAttributeEntry {ident, decorator: Some(decorator)})  
    }
    Ok(InArrayAttributeEntry {ident, decorator: None})
}