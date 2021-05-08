use crate::decl_fn_parsing::DeclaredFunction;
use crate::{DECL_FN_NAME, FIELD_SELECTOR_NAME};
use proc_macro2::TokenTree;
use proc_macro_error::*;
use std::collections::HashMap;

pub struct FieldParser<'a> {
    methods: &'a mut HashMap<syn::Ident, DeclaredFunction>,
    ident: &'a proc_macro2::Ident,
    include_ident: &'a proc_macro2::Ident,
    correct_fns: Vec<proc_macro2::Ident>,
    need_comma: bool,
    error: bool,
}

impl<'a> FieldParser<'a> {
    pub fn new(
        methods: &'a mut HashMap<syn::Ident, DeclaredFunction>,
        ident: &'a proc_macro2::Ident,
        include_ident: &'a proc_macro2::Ident,
    ) -> FieldParser<'a> {
        FieldParser {
            methods,
            ident,
            include_ident,
            correct_fns: vec![],
            need_comma: false,
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
                            "Method '{}' was not declared with the attribute '{}' at struct level.",
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
                abort!(self.include_ident.span(), "'{}' shouldn't contain those tokens.", FIELD_SELECTOR_NAME;
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
                _ => self.error = true,
            }
        }
    }

    fn parse_ident(&mut self, name: &proc_macro2::Ident) {
        if self.need_comma {
            self.error = true;
        }
        match self.methods.get_mut(&name) {
            Some(ref mut method) => {
                let ident = self.ident.clone();
                let has_this_field_already = method.fields.iter().any(|field| field == &ident);
                if has_this_field_already {
                    abort!(self.include_ident.span(), "Field '{}' is already included in method '{}', no need to repeat it.", self.ident, name;
                        help = "Remove the repeated entries.");
                }
                method.fields.push(ident);
                self.need_comma = true;
            }
            None => self.error = true,
        }
        self.correct_fns.push(name.clone());
    }

    fn parse_punct(&mut self, punct: &proc_macro2::Punct) {
        if self.need_comma && punct.to_string() == "," {
            self.need_comma = false;
        } else {
            self.error = true;
        }
    }
}

fn in_array_correct_syntax_without_name() -> String {
    format!("#[{}(your_generated_function_name)]", FIELD_SELECTOR_NAME)
}

fn in_array_correct_syntax(correct_fns: &str) -> String {
    format!("#[{}({})]", FIELD_SELECTOR_NAME, correct_fns)
}
