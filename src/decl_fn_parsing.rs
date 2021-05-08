use crate::{DECL_FN_NAME, DERIVE_NAME, IMPLICIT_SELECT_ALL_NAME};
use proc_macro2::TokenTree;
use proc_macro_error::*;
use quote::quote;

#[derive(Debug)]
pub struct DeclaredFunction {
    pub name: syn::Ident,
    pub vis: proc_macro2::TokenStream,
    pub ty: proc_macro2::TokenStream,
    pub body: proc_macro2::TokenStream,
    pub is_mut: bool,
    pub is_ref: bool,
    pub fields: Vec<syn::Ident>,
    pub implicit_select_all_tys: Vec<proc_macro2::TokenStream>,
}

pub fn parse_declared_method(
    tokens: proc_macro2::TokenStream,
    gen_array_span: proc_macro2::Span,
) -> DeclaredFunction {
    DeclaredMethodParser::new().parse(tokens, gen_array_span)
}

#[derive(Debug)]
enum FunctionParsing {
    Begin,
    ExpectingFnOrPubCrate,
    ExpectingFn,
    ExpectingName,
    ExpectingColon,
    ExpectingArrowEnd,
    ExpectingType,
    ExpectingImplicitSelectAllDecl,
    ExpectingImplicitSelectAllColon,
    ExpectingImplicitSelectAllTypes,
    Error,
}

struct DeclaredMethodParser {
    search_element: FunctionParsing,
    name: Option<syn::Ident>,
    ty: Vec<TokenTree>,
    vis: Vec<TokenTree>,
    body: Vec<TokenTree>,
    implicit_select_all_tys: Vec<Vec<TokenTree>>,
    implicit_select_all_cur_ty: usize,
    implicit_select_all_inserted_valid_comma: bool
}

impl DeclaredMethodParser {
    pub fn new() -> DeclaredMethodParser {
        DeclaredMethodParser {
            search_element: FunctionParsing::Begin,
            name: None,
            ty: vec![],
            vis: vec![],
            body: vec![],
            implicit_select_all_tys: vec![],
            implicit_select_all_cur_ty: 0,
            implicit_select_all_inserted_valid_comma: false
        }
    }
    pub fn parse(
        mut self,
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
            let decl_fn = DeclaredFunction {
                name,
                vis: self.vis.into_iter().collect(),
                ty: self.ty.into_iter().collect(),
                body: self.body.into_iter().collect(),
                is_mut,
                is_ref,
                fields: vec![],
                implicit_select_all_tys: self.implicit_select_all_tys.clone().into_iter().map(|ty| ty.into_iter().collect()).collect()
            };
            if !decl_fn.implicit_select_all_tys.is_empty() {
                let slice: Vec<String> = decl_fn.implicit_select_all_tys.iter().map(|tokens| tokens.to_string()).collect();
                if let Some(duplicated) = (1..slice.len()).position(|i| slice[i..].contains(&slice[i - 1])) {
                    abort!(self.implicit_select_all_tys[duplicated][0].span(), "'{}' tried to declare a method '{}' with a '{}' clause, but it contained a duplicated return type '{}'", DECL_FN_NAME, decl_fn.name, IMPLICIT_SELECT_ALL_NAME, slice[duplicated];
                        help = "Remove one of the '{}' from the '{}' clause", slice[duplicated], IMPLICIT_SELECT_ALL_NAME;);
                }
            }
            Some(decl_fn)
        } else {
            None
        };
        match self.search_element {
            FunctionParsing::ExpectingType | FunctionParsing::ExpectingImplicitSelectAllTypes => {
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
            match self.search_element {
                FunctionParsing::ExpectingType => match token {
                    TokenTree::Punct(comma) if comma.to_string() == "," => self.search_element = FunctionParsing::ExpectingImplicitSelectAllDecl,
                    _ => self.ty.push(token.clone()),
                },
                FunctionParsing::ExpectingImplicitSelectAllDecl => match token {
                    TokenTree::Ident(ref ident) if ident.to_string() == IMPLICIT_SELECT_ALL_NAME => self.search_element = FunctionParsing::ExpectingImplicitSelectAllColon,
                    _ => self.search_element = FunctionParsing::Error,
                },
                FunctionParsing::ExpectingImplicitSelectAllColon => match token {
                    TokenTree::Punct(colon) if colon.to_string() == ":" => self.search_element = FunctionParsing::ExpectingImplicitSelectAllTypes,
                    _ => self.search_element = FunctionParsing::Error,
                },
                FunctionParsing::ExpectingImplicitSelectAllTypes => match token {
                    TokenTree::Punct(comma) if comma.to_string() == "," && !self.implicit_select_all_inserted_valid_comma => {
                        self.implicit_select_all_inserted_valid_comma = true;
                        self.implicit_select_all_cur_ty += 1;
                    },
                    TokenTree::Punct(comma) if comma.to_string() == "," => self.search_element = FunctionParsing::Error,
                    _ => {
                        while self.implicit_select_all_tys.len() <= self.implicit_select_all_cur_ty {
                            self.implicit_select_all_tys.push(vec![]);
                        }
                        self.implicit_select_all_tys[self.implicit_select_all_cur_ty].push(token.clone());
                        self.implicit_select_all_inserted_valid_comma = false;
                    }
                },
                _ => {
                    match token {
                        TokenTree::Ident(ref ident) => self.parse_ident(ident, &token),
                        TokenTree::Group(_) => self.parse_group(&token),
                        TokenTree::Punct(ref punct) => self.parse_punct(punct),
                        _ => self.search_element = FunctionParsing::Error,
                    }
                }
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
            FunctionParsing::ExpectingFnOrPubCrate | FunctionParsing::ExpectingFn => {
                match ident.to_string().as_ref() {
                    "fn" => {
                        self.body.push(token.clone());
                        self.search_element = FunctionParsing::ExpectingName;
                    }
                    _ => self.search_element = FunctionParsing::Error,
                }
            }
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
