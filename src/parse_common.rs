use syn::parse::{ParseStream, Result};
use syn::{Token, Type};
use syn::{bracketed, AttrStyle, Attribute, Path};
use quote::quote;
use regex::Regex;

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

pub fn equal_types(field_ty: &Type, implicit_ty: &Type) -> bool {
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

    macro_rules! test_eq_types {
        ($($name:ident: $str1:expr, $str2:expr, $expected:expr)*) => {
        $(
            #[test]
            fn $name() {
                assert_eq!(
                    equal_types(
                        &syn::parse_str($str1).unwrap(),
                        &syn::parse_str($str2).unwrap()
                    ),
                    $expected
                );
            }
        )*
        }
    }

    test_eq_types! {
        equal_types_exactly_same_returns_true: "Option<i32>", "Option<i32>", true
        equal_types_compared_to_wildcard_returns_true: "Option<i32>", "_", true
        equal_types_compared_to_option_wildcard_returns_true: "Option<i32>", "Option<_>", true
        equal_types_exactly_same_returns_true2: "Option<Result<(i32, f32), std::fmt::Error>>", "Option<Result<(i32, f32), std::fmt::Error>>", true
        equal_types_different1_returns_false: "Option<Result<(i32, f32), std::fmt::Error>>", "Option<Result<(i32, i32), std::fmt::Error>>", false
        equal_types_different2_returns_false: "Option<Result<(i32, f32), std::error::Error>>", "Option<Result<(i32, f32), std::fmt::Error>>", false
        equal_types_compared_to_wildcard3_returns_true: "Option<Option<i32>>", "Option<Option<_>>", true
        equal_types_compared_to_wildcard4_returns_true: "Option<Result<i32, i32>>", "Option<Result<_, i32>>", true
        equal_types_compared_to_wildcard5_returns_false: "Option<Result<i32, i32>>", "Option<Result<_, f32>>", false
    }
}