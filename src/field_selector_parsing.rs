use crate::decl_fn_parsing::{CastKind, DeclaredFunction};
use crate::{DECL_FN_NAME, FIELD_SELECTOR_NAME};
use proc_macro2::Delimiter;
use proc_macro2::TokenTree;
use proc_macro_error::*;
use std::collections::HashMap;

pub struct FieldParser<'a> {
    methods: &'a mut HashMap<syn::Ident, DeclaredFunction>,
    ident: &'a proc_macro2::Ident,
    ty: &'a syn::Type,
    include_ident: &'a proc_macro2::Ident,
    correct_fns: Vec<proc_macro2::Ident>,
    need_comma: bool,
    parsed_ident: bool,
    previous_method: Option<proc_macro2::Ident>,
    error: bool,
}

impl<'a> FieldParser<'a> {
    pub fn new(
        methods: &'a mut HashMap<syn::Ident, DeclaredFunction>,
        ident: &'a proc_macro2::Ident,
        ty: &'a syn::Type,
        include_ident: &'a proc_macro2::Ident,
    ) -> FieldParser<'a> {
        FieldParser {
            methods,
            ident,
            ty,
            include_ident,
            correct_fns: vec![],
            previous_method: None,
            need_comma: false,
            parsed_ident: false,
            error: false,
        }
    }

    pub fn parse(mut self, tokens: proc_macro2::TokenStream) {
        for token in tokens {
            match token {
                TokenTree::Group(group) => self.parse_group(group),
                _ => self.error = true,
            }
        }
        if self.error {
            if !self.correct_fns.is_empty() {
                for correct_fn in &self.correct_fns {
                    if self.methods.get_mut(&correct_fn).is_none() {
                        abort!(
                            correct_fn.span(),
                            "Method '{}' was not declared with the attribute '{}' at struct level",
                            correct_fn,
                            DECL_FN_NAME
                        );
                    }
                }
                let correct_fns = self
                    .correct_fns
                    .iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                abort!(self.include_ident.span(), "'{}' shouldn't contain these tokens.", FIELD_SELECTOR_NAME;
                    help = "Correct syntax is {}", in_array_correct_syntax(&correct_fns));
            } else {
                abort!(self.include_ident.span(), "'{}' was used with the wrong syntax.", FIELD_SELECTOR_NAME;
                    help = "Correct syntax is {}", in_array_correct_syntax_without_name());
            }
        }
    }

    fn parse_group(&mut self, group: proc_macro2::Group) {
        for token in group.stream().into_iter() {
            match token {
                TokenTree::Ident(name) => self.parse_ident(&name),
                TokenTree::Punct(punct) => self.parse_punct(&punct),
                TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                    self.parse_decorators(group)
                }
                _ => self.error = true,
            }
        }
    }

    fn parse_decorators(&mut self, group: proc_macro2::Group) {
        if !self.parsed_ident {
            abort!(group.span(), "'{}' decorators have to be placed after a method name", FIELD_SELECTOR_NAME;
                help = "Correct syntax is {}", in_array_correct_syntax_with_decorators_without_name());
        }
        for token in group.stream().into_iter() {
            match token {
                TokenTree::Ident(name) => {
                    let cast_kind = match name.to_string().as_ref() {
                        "cast" => CastKind::SafeCast,
                        "unsafe_transmute" => CastKind::UnsafeTransmute,
                        _ => {
                            abort!(group.span(), "'{}' doesn't allow '{}' as a decorator.", FIELD_SELECTOR_NAME, name;
                            help = "Check the documentation about decorators")
                        }
                    };
                    let method_name = match self.previous_method {
                        Some(ref method_name) => method_name,
                        None => {
                            abort!(group.span(), "'{}' can not decorate the given method because it wasn't declared with '{}'", FIELD_SELECTOR_NAME, DECL_FN_NAME;
                            help = "Correct syntax is {}", in_array_correct_syntax_with_decorators_without_name())
                        }
                    };
                    match self.methods.get_mut(method_name) {
                        Some(ref mut method) => method.casts.push((
                            self.ident.clone(),
                            self.ty.clone(),
                            method.ty.clone(),
                            cast_kind,
                        )),
                        None => {
                            abort!(group.span(), "'{}' can not decorate method '{}' because it wasn't found", FIELD_SELECTOR_NAME, method_name;
                            help = "Correct syntax is {}", in_array_correct_syntax_with_decorators(&method_name.to_string()))
                        }
                    }
                }
                _ => {
                    abort!(group.span(), "'{}' doesn't allow '{}' as a decorator.", FIELD_SELECTOR_NAME, token.to_string();
                        help = "Check the documentation about decorators")
                }
            }
        }
        self.parsed_ident = false;
    }

    fn parse_ident(&mut self, name: &proc_macro2::Ident) {
        self.parsed_ident = true;
        if self.need_comma {
            self.error = true;
        }
        match self.methods.get_mut(&name) {
            Some(ref mut method) => {
                let ident = self.ident.clone();
                let has_this_field_already = method.fields.iter().any(|field| field == &ident);
                if has_this_field_already {
                    abort!(self.include_ident.span(), "Field '{}' is already included in method '{}', no need to repeat it.", self.ident, name;
                        help = "Remove the repeated entries");
                }
                self.previous_method = Some(name.clone());
                method.fields.push(ident);
                self.need_comma = true;
            }
            None => self.error = true,
        }
        self.correct_fns.push(name.clone());
    }

    fn parse_punct(&mut self, punct: &proc_macro2::Punct) {
        self.parsed_ident = false;
        if self.need_comma && punct.to_string() == "," {
            self.need_comma = false;
            self.previous_method = None;
        } else {
            self.error = true;
        }
    }
}

fn in_array_correct_syntax(correct_fns: &str) -> String {
    format!("#[{}({})]", FIELD_SELECTOR_NAME, correct_fns)
}

fn in_array_correct_syntax_without_name() -> String {
    in_array_correct_syntax("your_generated_function_name")
}

fn in_array_correct_syntax_with_decorators(correct_fns: &str) -> String {
    format!(
        "#[{}({} {{ decorators }})]",
        FIELD_SELECTOR_NAME, correct_fns
    )
}

fn in_array_correct_syntax_with_decorators_without_name() -> String {
    in_array_correct_syntax_with_decorators("your_generated_function_name")
}
