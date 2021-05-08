use crate::decl_fn_parsing::{parse_declared_method, DeclaredFunction};
use crate::field_selector_parsing::FieldParser;
use crate::{DECL_FN_NAME, FIELD_SELECTOR_NAME};
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use proc_macro_error::*;
use quote::quote;
use std::collections::HashMap;

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
        let methods = self.read_ast();
        if methods.is_empty() {
            return quote! {}.into();
        }

        let impl_fns = make_impl_fns(methods);

        let (ty, generics) = (&self.ast.ident, &self.ast.generics);
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let tokens = quote! {
            impl #impl_generics #ty #ty_generics
                #where_clause
            {
                #(#impl_fns)
                *
            }
        };
        tokens.into()
    }

    fn read_ast(&self) -> HashMap<syn::Ident, DeclaredFunction> {
        let mut methods = read_methods(self.ast.attrs.clone());

        if methods.is_empty() {
            emit_warning!(
                self.ast.ident.span(),
                "The type '{}' derives '{}' but does not contain any '{}' attribute, so '{}' does nothing.",
                self.ast.ident,
                self.derive_name,
                DECL_FN_NAME,
                self.derive_name
            );
        }

        match self.ast.data {
            syn::Data::Struct(ref class) => read_fields(&class.fields, &mut methods),
            _ => abort!(
                self.ast.ident.span(),
                "The type '{}' is not a struct but tries to derive '{}' which can only be used on structs.",
                self.ast.ident,
                self.derive_name
            ),
        }
        methods
    }
}

fn read_methods(attrs: Vec<syn::Attribute>) -> HashMap<syn::Ident, DeclaredFunction> {
    attrs
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
        })
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

fn read_attr_ident(
    attr: &syn::Attribute,
    ident: &proc_macro2::Ident,
    methods: &mut HashMap<syn::Ident, DeclaredFunction>,
) {
    let segments: Vec<_> = attr
        .path
        .segments
        .iter()
        .filter_map(|segment| {
            if segment.ident == FIELD_SELECTOR_NAME {
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
            FIELD_SELECTOR_NAME
        ),
    };

    FieldParser::new(methods, ident, include_ident).parse(attr.tokens.clone());
}

fn make_impl_fns(methods: HashMap<syn::Ident, DeclaredFunction>) -> Vec<TokenTree> {
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
        })
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
