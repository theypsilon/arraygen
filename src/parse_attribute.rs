use syn::parse::{ParseStream, Result};
use syn::Token;
use syn::{bracketed, AttrStyle, Attribute, Path};

pub fn single_parse_outer_attribute(input: ParseStream) -> Result<()> {
    let content;
    let _ = Attribute {
        pound_token: input.parse()?,
        style: AttrStyle::Outer,
        bracket_token: bracketed!(content in input),
        path: content.call(Path::parse_mod_style)?,
        tokens: content.parse()?,
    };
    Ok(())
}

pub fn parse_inner_attributes(input: ParseStream) -> Result<()> {
    while input.peek(Token![#]) && input.peek(Token![!]) {
        input.call(single_parse_inner_attribute)?;
    }
    Ok(())
}

fn single_parse_inner_attribute(input: ParseStream) -> Result<()> {
    let content;
    let _ = Attribute {
        pound_token: input.parse()?,
        style: AttrStyle::Inner(input.parse()?),
        bracket_token: bracketed!(content in input),
        path: content.call(Path::parse_mod_style)?,
        tokens: content.parse()?,
    };
    Ok(())
}
