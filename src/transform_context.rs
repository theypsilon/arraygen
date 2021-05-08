use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use proc_macro_error::*;
use quote::quote;
use std::collections::HashMap;
use crate::{DERIVE_NAME, DECL_FN_NAME, INCLUDE_FIELD};

pub struct TransformContext {
    ast: syn::DeriveInput,
    derive_name: &'static str,
}

impl TransformContext {
    pub fn new(input: TokenStream, derive_name: &'static str) -> TransformContext {
        TransformContext {
            ast: syn::parse(input).expect("Could not parse AST."),
            derive_name,
        }
    }

    pub fn transform_ast(&self) -> TokenStream {
        let mut methods = self
            .ast
            .attrs
            .clone()
            .into_iter()
            .flat_map(|attr| {
                let attribute = attr.clone();
                attr.path.segments.into_iter().filter_map(move |segment| {
                    if segment.ident == DECL_FN_NAME {
                        Some((attribute.clone(), segment.ident.span()))
                    } else {
                        None
                    }
                })
            })
            .map(|(attr, span)| parse_declared_method(attr.tokens, span))
            .fold(HashMap::new(), |mut acc, method| {
                if acc.contains_key(&method.name) {
                    abort!(
                        method.name.span(),
                        "{} found two or more methods declared with the name '{}'.",
                        DECL_FN_NAME,
                        method.name
                    )
                } else {
                    acc.insert(method.name.clone(), method);
                    acc
                }
            });

        match self.ast.data {
            syn::Data::Struct(ref class) => read_fields(&class.fields, &mut methods),
            _ => abort!(
                self.ast.ident.span(),
                "The type '{}' is not a struct but tries to derive '{}' which can only be used on structs.",
                self.ast.ident,
                self.derive_name
            ),
        }

        if methods.is_empty() {
            emit_warning!(
                self.ast.ident.span(),
                "The type '{}' derives '{}' but does not contain any '{}' attribute, so '{}' does nothing.",
                self.ast.ident,
                self.derive_name,
                DECL_FN_NAME,
                self.derive_name
            );
            return quote! {}.into();
        }

        let methods =
            methods
                .into_iter()
                .fold(Vec::<TokenTree>::new(), |mut acc, (name, method)| {
                    if method.fields.is_empty() {
                        emit_warning!(
                            method.name.span(),
                            "Method '{}' returns an empty array.",
                            name
                        );
                    }
                    acc.extend(make_method_tokens(&method));
                    acc
                });

        let (ty, generics) = (&self.ast.ident, &self.ast.generics);
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let tokens = quote! {
            impl #impl_generics #ty #ty_generics
                #where_clause
            {
                #(#methods)
                *
            }
        };
        tokens.into()
    }
}

#[derive(Debug)]
struct DeclaredFunction {
    name: syn::Ident,
    vis: proc_macro2::TokenStream,
    ty: proc_macro2::TokenStream,
    body: proc_macro2::TokenStream,
    is_mut: bool,
    is_ref: bool,
    fields: Vec<syn::Ident>,
}

enum FunctionParsing {
    Begin,
    ExpectingFnOrPubCrate,
    ExpectingFn,
    ExpectingName,
    ExpectingColon,
    ExpectingArrowEnd,
    ExpectingType,
    Error,
}

struct DeclaredMethodParser {
    search_element: FunctionParsing,
    name: Option<syn::Ident>,
    ty: Vec<TokenTree>,
    vis: Vec<TokenTree>,
    body: Vec<TokenTree>
}

impl DeclaredMethodParser {
    pub fn new() -> DeclaredMethodParser {
        DeclaredMethodParser {
            search_element: FunctionParsing::Begin,
            name: None,
            ty: vec![],
            vis: vec![],
            body: vec![],
        }
    }
    pub fn parse(mut self,
        tokens: proc_macro2::TokenStream,
        gen_array_span: proc_macro2::Span,
    ) -> DeclaredFunction {
        self.parse_tokens(tokens);
        if self.ty.is_empty() {
            self.search_element = FunctionParsing::Error;
        }
        let is_ref = !self.ty.is_empty() && self.ty[0].to_string() == "&";
        let is_mut = is_ref && self.ty.len() >= 2 && self.ty[1].to_string() == "mut";
        let decl_fn = if let Some(name) = self.name {
            Some(DeclaredFunction {
                name,
                vis: self.vis.into_iter().collect(),
                ty: self.ty.into_iter().collect(),
                body: self.body.into_iter().collect(),
                is_mut,
                is_ref,
                fields: vec![],
            })
        } else {
            None
        };
        match self.search_element {
            FunctionParsing::ExpectingType => {
                if let Some(decl_fn) = decl_fn {
                    return decl_fn;
                }
            }
            FunctionParsing::Error => {
                if let Some(decl_fn) = decl_fn {
                    abort!(decl_fn.name.span(), "'{}' tried to declare a method '{}', but the return type syntax was wrong.", DECL_FN_NAME, decl_fn.name;
                        help = "Correct syntax is {}", decl_fn_correct_syntax(&decl_fn););
                } else {
                    abort!(gen_array_span, "'{}' was used with the wrong syntax.", DECL_FN_NAME;
                        help = "Correct syntax is {}", decl_fn_correct_syntax_without_name());
                }
            }
            _ => {}
        }
        abort!(
            gen_array_span,
            "Bug on '{}', contact with the maintainer of {} crate.",
            DECL_FN_NAME,
            DERIVE_NAME
        );
    }
    fn parse_tokens(&mut self, tokens: proc_macro2::TokenStream) {
        for token in tokens.into_iter() {
            match token {
                TokenTree::Group(group) => self.parse_main_group(group),
                _ => self.search_element = FunctionParsing::Error,
            }
        }
    }
    fn parse_main_group(&mut self, group: proc_macro2::Group) {
        for token in group.stream().into_iter() {
            if let FunctionParsing::ExpectingType = self.search_element {
                self.ty.push(token.clone());
                continue;
            }
            match token {
                TokenTree::Ident(ref ident) => self.parse_ident(ident, &token),
                TokenTree::Group(_) => self.parse_group(&token),
                TokenTree::Punct(ref punct) => self.parse_punct(punct),
                _ => self.search_element = FunctionParsing::Error,
            }
        }
    }
    fn parse_ident(&mut self, ident: &proc_macro2::Ident, token: &proc_macro2::TokenTree) {
        match self.search_element {
            FunctionParsing::Begin => match ident.to_string().as_ref() {
                "pub" => {
                    self.vis.push(token.clone());
                    self.search_element = FunctionParsing::ExpectingFnOrPubCrate;
                }
                "fn" => {
                    self.body.push(token.clone());
                    self.search_element = FunctionParsing::ExpectingName;
                }
                _ => self.search_element = FunctionParsing::Error,
            },
            FunctionParsing::ExpectingFnOrPubCrate
            | FunctionParsing::ExpectingFn => match ident.to_string().as_ref() {
                "fn" => {
                    self.body.push(token.clone());
                    self.search_element = FunctionParsing::ExpectingName;
                }
                _ => self.search_element = FunctionParsing::Error,
            },
            FunctionParsing::ExpectingName => {
                self.name = Some(ident.clone());
                self.body.push(token.clone());
                self.search_element = FunctionParsing::ExpectingColon;
            }
            _ => self.search_element = FunctionParsing::Error,
        }
    }
    fn parse_group(&mut self, token: &proc_macro2::TokenTree) {
        match self.search_element {
            FunctionParsing::ExpectingFnOrPubCrate => {
                self.vis.push(token.clone());
                self.search_element = FunctionParsing::ExpectingFn;
            }
            _ => self.search_element = FunctionParsing::Error,
        }
    }
    fn parse_punct(&mut self, punct: &proc_macro2::Punct) {
        match self.search_element {
            FunctionParsing::ExpectingArrowEnd => {
                if punct.to_string() == ">" {
                    self.search_element = FunctionParsing::ExpectingType;
                } else {
                    self.search_element = FunctionParsing::Error;
                }
            }
            FunctionParsing::ExpectingColon => {
                if punct.to_string() == ":" {
                    self.search_element = FunctionParsing::ExpectingType;
                } else if punct.to_string() == "-" {
                    self.search_element = FunctionParsing::ExpectingArrowEnd;
                } else {
                    self.search_element = FunctionParsing::Error
                }
            }
            _ => self.search_element = FunctionParsing::Error,
        }
    }
}

fn parse_declared_method(
    tokens: proc_macro2::TokenStream,
    gen_array_span: proc_macro2::Span,
) -> DeclaredFunction {
    DeclaredMethodParser::new().parse(tokens, gen_array_span)
}

fn read_fields(fields: &syn::Fields, methods: &mut HashMap<syn::Ident, DeclaredFunction>) {
    for field in fields.iter() {
        if field.attrs.is_empty() {
            continue;
        }
        if let Some(ref ident) = field.ident {
            for attr in field.attrs.iter() {
                read_attr_ident(&attr, ident, methods);
            }
        }
    }
}

struct FieldParser<'a> {
    methods: &'a mut HashMap<syn::Ident, DeclaredFunction>,
    ident: &'a proc_macro2::Ident,
    include_ident: &'a proc_macro2::Ident,
    correct_fns: Vec<proc_macro2::Ident>,
    need_comma: bool,
    error: bool,
}

impl<'a> FieldParser<'a> {
    pub fn new(methods: &'a mut HashMap<syn::Ident, DeclaredFunction>, ident: &'a proc_macro2::Ident, include_ident: &'a proc_macro2::Ident) -> FieldParser<'a> {
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
                        abort!(correct_fn.span(), "Method '{}' was not declared with the attribute '{}' at struct level.", correct_fn, DECL_FN_NAME);
                    }
                }
                let correct_fns = self.correct_fns
                    .iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                abort!(self.include_ident.span(), "'{}' shouldn't contain those tokens.", INCLUDE_FIELD;
                    help = "Correct syntax is {}", include_field_correct_syntax(&correct_fns));
            } else {
                abort!(self.include_ident.span(), "'{}' was used with the wrong syntax.", INCLUDE_FIELD;
                    help = "Correct syntax is {}", include_field_correct_syntax_without_name());
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

fn read_attr_ident(attr: &syn::Attribute, ident: &proc_macro2::Ident, methods: &mut HashMap<syn::Ident, DeclaredFunction>) {
    let segments: Vec<_> = attr
        .path
        .segments
        .iter()
        .filter_map(|segment| {
            if segment.ident == INCLUDE_FIELD {
                Some(segment.ident.clone())
            } else {
                None
            }
        })
        .collect();
    let include_ident = match segments.len() {
        0 => return,
        1 => &segments[0],
        // @TODO Not sure if this condition can actually happen, not covered in tests yet.
        _ => abort!(
            segments[0].span(),
            "Wrong syntax, used multiple '{}' in same attribute.",
            INCLUDE_FIELD
        ),
    };

    FieldParser::new(methods, ident, include_ident).parse(attr.tokens.clone());
}

fn make_method_tokens(props: &DeclaredFunction) -> proc_macro2::TokenStream {
    let field_idents = &props.fields;
    let count = field_idents.len();
    let return_type = &props.ty;
    let vis = &props.vis;
    let body = &props.body;
    let refa = if props.is_ref {
        if props.is_mut {
            quote! {&mut}
        } else {
            quote! {&}
        }
    } else {
        quote! {}
    };
    let muta = if props.is_mut {
        quote! {mut}
    } else {
        quote! {}
    };
    quote! {
        #[inline(always)]
        #vis #body (& #muta self) -> [#return_type; #count] {
            [#(#refa self.#field_idents),*]
        }
    }
}

fn decl_fn_correct_syntax_without_name() -> String {
    format!("#[{}(fn your_function_name: YourReturnType)]", DECL_FN_NAME)
}

fn decl_fn_correct_syntax(decl_fn: &DeclaredFunction) -> String {
    let vis = &decl_fn.vis;
    let body = &decl_fn.body;
    let signature = quote! {
        #vis #body: YourReturnType
    };
    format!("#[{}({})]", DECL_FN_NAME, signature.to_string())
}

fn include_field_correct_syntax_without_name() -> String {
    format!("#[{}(your_generated_function_name)]", INCLUDE_FIELD)
}

fn include_field_correct_syntax(correct_fns: &str) -> String {
    format!("#[{}({})]", INCLUDE_FIELD, correct_fns)
}
