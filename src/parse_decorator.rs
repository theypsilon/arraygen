use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token;
use syn::{braced, Error, Ident, Token};

use crate::FIELD_SELECTOR_NAME;

#[derive(Clone, PartialEq)]
pub enum CastKind {
    SafeCast,
    UnsafeTransmute,
}

pub struct Decorator {
    pub override_implicit: bool,
    pub cast: Option<CastKind>,
}

impl Decorator {
    pub fn new() -> Decorator {
        Decorator {
            override_implicit: false,
            cast: None,
        }
    }
}

impl Parse for Decorator {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut decorator = Decorator::new();
        if input.peek(token::Brace) {
            let content;
            let _ = braced!(content in input);
            for ident in Punctuated::<Ident, Token![,]>::parse_separated_nonempty(&content)?.iter() {
                match ident.to_string().as_ref() {
                    "override_implicit" if !decorator.override_implicit => {
                        decorator.override_implicit = true
                    }
                    "cast" if decorator.cast.is_none() => decorator.cast = Some(CastKind::SafeCast),
                    "unsafe_transmute" if decorator.cast.is_none() => {
                        decorator.cast = Some(CastKind::UnsafeTransmute)
                    }
                    decorator => {
                        return Err(Error::new_spanned(
                            ident,
                            format!(
                                "{} doesn't allow '{}' as decorator here",
                                FIELD_SELECTOR_NAME, decorator
                            ),
                        ))
                    }
                };
            }
        }
        Ok(decorator)
    }
}
