//!
//! This crate provides `Arraygen` derive macro for structs, which generates methods returning arrays filled with the selected struct fields.
//!
//! Complete example:
//! 
//! ```rust
//! use arraygen::Arraygen;
//! 
//! #[derive(Arraygen, Debug)]
//! #[gen_array(fn get_names: &mut String)]
//! struct Person {
//!     #[in_array(get_names)]
//!     first_name: String,
//!     #[in_array(get_names)]
//!     last_name: String,
//! }
//! 
//! let mut person = Person { first_name: "Ada".into(), last_name: "Lovelace".into() };
//! for name in person.get_names().iter_mut() {
//!     **name = name.to_lowercase();
//! }
//! 
//! assert_eq!(format!("{:?}", person), "Person { first_name: \"ada\", last_name: \"lovelace\" }");
//! ```
//!
//! As you can see above, the attribute `gen_array` generates a new method returning an array of the given type. And the attribute `in_array` selects those struct fields to be used by that method.
//! 
//! What `Arraygen` does under the hood is simply generating the following impl:
//! 
//! ```rust
//! struct Person {
//!     first_name: String,
//!     last_name: String,
//! }
//! impl Person {
//!     #[inline(always)]
//!     fn get_names(&mut self) -> [&mut String; 2] {
//!         [&mut self.first_name, &mut self.last_name]
//!     }
//! }
//! ```

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use proc_macro_error::*;
use quote::quote;
use std::collections::HashMap;

/// The `Arraygen` derive allows you to use the attribute `gen_array` at the struct level and the attribute `in_array` in each contained field.
/// 
/// With `gen_array` you can declare your new methods returning arrays in the following way:
///
/// `#[gen_array(?visibility fn your_method_name: YourReturnType)]`
/// 
/// * 'visibility' is optional, you can let it blank entirely, or write `pub`, `pub(crate)` an any other pub variant.
/// * 'your_method_name' can be any valid method name, you can't use a name taken by another mehtod in the struct impl, including also other methods generated by `gen_array`.
/// * 'YourReturnType' can be any Rust type that can appear in a struct field. Notice that if the `type` does not implement the trait 'Copy', you need to return `&type` or `&mut type` instead in order to avoid ownership errors.
/// 
/// There is no limit in the number of methods you can generate.
/// 
/// With `in_array` you select which field is returned by which method generated by `gen_array`.
/// Sintax is the following one:
/// 
/// `#[in_array(your_method_name)]`
/// 
/// * `your_method_name` needs to be some method name generated by `gen_array`.
/// 
/// The only thing you need to care is that the type returned by `your_method_name` is compatible with the type of the field.
/// Notice that non-reference field types can be returned as references, but not the other way around.
/// 
/// Is also good to know that the same field can be included in many generated methods, like in the following example:
/// 
/// ```rust
/// use arraygen::Arraygen;
/// 
/// #[derive(Arraygen)]
/// #[gen_array(fn odds: i32)]
/// #[gen_array(fn evens: i32)]
/// #[gen_array(fn primes: i32)]
/// struct Numbers {
///     #[in_array(odds)]
///     one: i32,
/// 
///     #[in_array(evens)]
///     #[in_array(primes)]
///     two: i32,
/// 
///     #[in_array(odds)]
///     #[in_array(primes)]
///     three: i32,
/// 
///     #[in_array(evens)]
///     four: i32,
/// 
///     #[in_array(primes)]
///     #[in_array(odds)]
///     five: i32
/// }
/// 
/// let numbers = Numbers { one: 1, two: 2, three: 3, four: 4, five: 5};
/// assert_eq!(numbers.odds(), [1, 3, 5]);
/// assert_eq!(numbers.evens(), [2, 4]);
/// assert_eq!(numbers.primes(), [2, 3, 5]);
/// ```
/// 
/// # Trait Objects
/// 
/// A very good use of this derive is being able to extract trait objects from different concrete types, so you can operate in all of them at once.
/// 
/// ```rust
/// use arraygen::Arraygen;
/// 
/// trait Animal {
///     fn talk(&self) -> &'static str;
/// }
/// 
/// struct Dog {}
/// impl Animal for Dog {
///     fn talk(&self) -> &'static str {
///         "bark"
///     }
/// }
/// 
/// struct Cat {}
/// impl Animal for Cat {
///     fn talk(&self) -> &'static str {
///         "meow"
///     }
/// }
/// 
/// #[derive(Arraygen)]
/// #[gen_array(fn get_animals: &dyn Animal)]
/// struct Animals {
///     #[in_array(get_animals)]
///     dogo: Dog,
///     #[in_array(get_animals)]
///     tiger: Cat,
///     #[in_array(get_animals)]
///     kitty: Cat,
/// }
/// 
/// let animals = Animals { dogo: Dog {}, tiger: Cat {}, kitty: Cat {} };
/// let talk: Vec<&'static str> = animals.get_animals().iter().map(|animal| animal.talk()).collect();
/// assert_eq!(talk, ["bark", "meow", "meow"]);
/// ```
///

const DERIVE_NAME: &'static str = "Arraygen";
const DECL_FN_NAME: &'static str = "gen_array";
const INCLUDE_FIELD: &'static str = "in_array";

#[proc_macro_error]
#[proc_macro_derive(Arraygen, attributes(gen_array, in_array))]
pub fn arraygen(input: TokenStream) -> TokenStream {
    ImplContext::new(input, DERIVE_NAME).transform_ast()
}

struct ImplContext {
    ast: syn::DeriveInput,
    derive_name: &'static str,
}

impl ImplContext {
    fn new(input: TokenStream, derive_name: &'static str) -> ImplContext {
        ImplContext {
            ast: syn::parse(input).expect("Could not parse AST."),
            derive_name,
        }
    }

    fn transform_ast(&self) -> TokenStream {
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

        if methods.len() == 0 {
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
                    if method.fields.len() == 0 {
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

fn parse_declared_method(
    tokens: proc_macro2::TokenStream,
    gen_array_span: proc_macro2::Span,
) -> DeclaredFunction {
    let mut search_element = FunctionParsing::Begin;
    let mut name: Option<syn::Ident> = None;
    let mut ty: Vec<TokenTree> = vec![];
    let mut vis: Vec<TokenTree> = vec![];
    let mut body: Vec<TokenTree> = vec![];
    for token in tokens.into_iter() {
        match token {
            TokenTree::Group(group) => {
                for token in group.stream().into_iter() {
                    if let FunctionParsing::ExpectingType = search_element {
                        ty.push(token.clone());
                        continue;
                    }
                    match token {
                        TokenTree::Ident(ref ident) => match search_element {
                            FunctionParsing::Begin => match ident.to_string().as_ref() {
                                "pub" => {
                                    vis.push(token.clone());
                                    search_element = FunctionParsing::ExpectingFnOrPubCrate;
                                }
                                "fn" => {
                                    body.push(token.clone());
                                    search_element = FunctionParsing::ExpectingName;
                                }
                                _ => search_element = FunctionParsing::Error,
                            },
                            FunctionParsing::ExpectingFnOrPubCrate
                            | FunctionParsing::ExpectingFn => match ident.to_string().as_ref() {
                                "fn" => {
                                    body.push(token.clone());
                                    search_element = FunctionParsing::ExpectingName;
                                }
                                _ => search_element = FunctionParsing::Error,
                            },
                            FunctionParsing::ExpectingName => {
                                name = Some(ident.clone());
                                body.push(token.clone());
                                search_element = FunctionParsing::ExpectingColon;
                            }
                            _ => search_element = FunctionParsing::Error,
                        },
                        TokenTree::Group(_) => match search_element {
                            FunctionParsing::ExpectingFnOrPubCrate => {
                                vis.push(token.clone());
                                search_element = FunctionParsing::ExpectingFn;
                            }
                            _ => search_element = FunctionParsing::Error,
                        },
                        TokenTree::Punct(ref punct) => match search_element {
                            FunctionParsing::ExpectingArrowEnd => {
                                if punct.to_string() == ">" {
                                    search_element = FunctionParsing::ExpectingType;
                                } else {
                                    search_element = FunctionParsing::Error;
                                }
                            }
                            FunctionParsing::ExpectingColon => {
                                if punct.to_string() == ":" {
                                    search_element = FunctionParsing::ExpectingType;
                                } else if punct.to_string() == "-" {
                                    search_element = FunctionParsing::ExpectingArrowEnd;
                                } else {
                                    search_element = FunctionParsing::Error
                                }
                            }
                            _ => search_element = FunctionParsing::Error,
                        },
                        _ => search_element = FunctionParsing::Error,
                    }
                }
            }
            _ => search_element = FunctionParsing::Error,
        }
    }
    if ty.len() == 0 {
        search_element = FunctionParsing::Error;
    }
    let is_ref = ty.len() >= 1 && ty[0].to_string() == "&";
    let is_mut = is_ref && ty.len() >= 2 && ty[1].to_string() == "mut";
    let decl_fn = if let Some(name) = name {
        Some(DeclaredFunction {
            name,
            vis: vis.into_iter().collect(),
            ty: ty.into_iter().collect(),
            body: body.into_iter().collect(),
            is_mut,
            is_ref,
            fields: vec![],
        })
    } else {
        None
    };
    match search_element {
        FunctionParsing::ExpectingType => {
            if let Some(decl_fn) = decl_fn {
                return decl_fn;
            }
        }
        FunctionParsing::Error => {
            if let Some(decl_fn) = decl_fn {
                abort!(decl_fn.name.span(), "'{}' tried to declare a method '{}', but the return type syntax was wrong.", DECL_FN_NAME, decl_fn.name;
                    help = "Correct syntax is '{}'", decl_fn_correct_syntax(&decl_fn););
            } else {
                abort!(gen_array_span, "'{}' was used with the wrong syntax.", DECL_FN_NAME;
                    help = "Correct syntax is '{}'", decl_fn_correct_syntax_without_name());
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

fn read_fields(fields: &syn::Fields, methods: &mut HashMap<syn::Ident, DeclaredFunction>) {
    for field in fields.iter() {
        if field.attrs.is_empty() {
            continue;
        }
        if let Some(ref ident) = field.ident {
            for attr in field.attrs.iter() {
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
                    0 => continue,
                    1 => &segments[0],
                    _ => abort!(
                        segments[0].span(),
                        "Wrong syntax, used multiple '{}' in same attribute.",
                        INCLUDE_FIELD
                    ),
                };
                let mut error = false;
                let mut correct_fn = None;
                for token in attr.tokens.clone() {
                    match token {
                        TokenTree::Group(group) => {
                            for token in group.stream().into_iter() {
                                if let TokenTree::Ident(name) = token {
                                    match methods.get_mut(&name) {
                                        Some(ref mut method) => {
                                            if let None = correct_fn {
                                                method.fields.push(ident.clone());
                                            } else {
                                                error = true;
                                            }
                                        }
                                        None => error = true,
                                    }
                                    correct_fn = Some(name.clone());
                                } else {
                                    error = true;
                                }
                            }
                        }
                        _ => error = true,
                    }
                }
                if error {
                    if let Some(correct_fn) = correct_fn {
                        if let None = methods.get_mut(&correct_fn) {
                            abort!(correct_fn.span(), "Method '{}' was not declared with the attribute '{}' at struct level.", correct_fn, DECL_FN_NAME);
                        } else {
                            abort!(include_ident.span(), "Wrong syntax in '{}'.", INCLUDE_FIELD;
                                help = "Correct syntax is '{}'", include_field_correct_syntax(&correct_fn));
                        }
                    } else {
                        abort!(include_ident.span(), "Wrong syntax in '{}'.", INCLUDE_FIELD;
                            help = "Correct syntax is '{}'", include_field_correct_syntax_without_name());
                    }
                }
            }
        }
    }
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

fn include_field_correct_syntax(correct_fn: &syn::Ident) -> String {
    format!("#[{}({})]", INCLUDE_FIELD, correct_fn)
}
