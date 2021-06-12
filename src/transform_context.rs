use crate::casting::CastKind;
use crate::parse_derive_arraygen::DeriveArraygen;
use crate::parse_gen_array::GenArray;
use crate::{DECL_FN_NAME, DERIVE_NAME};
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use std::collections::HashMap;
use syn::parse_macro_input;
use syn::{Ident, Type};

pub fn transform_ast(input: TokenStream) -> TokenStream {
    let arraygen = parse_macro_input!(input as DeriveArraygen);

    let struct_name = arraygen.struct_name;

    if arraygen.gen_arrays.is_empty() {
        eprintln!(
            //struct_name.span(), @TODO emit warning
            "warning: The struct '{}' derives '{}' but does not contain any '{}' attribute, so '{}' does nothing.",
            struct_name,
            DERIVE_NAME,
            DECL_FN_NAME,
            DERIVE_NAME
        );
    }

    let impl_fns = make_impl_fns(arraygen.gen_arrays, &struct_name);
    let (impl_generics, ty_generics, where_clause) = arraygen.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics #struct_name #ty_generics
            #where_clause
        {
            #(#impl_fns)
            *
        }
    };

    tokens.into()
}

fn make_impl_fns(methods: HashMap<Ident, GenArray>, struct_name: &Ident) -> Vec<TokenTree> {
    methods
        .into_iter()
        .fold(Vec::<TokenTree>::new(), |mut acc, (name, method)| {
            if method.fields.is_empty() {
                eprintln!(
                    //method.fn_name.span(), @TODO emit warning
                    "warning: Method '{}' from struct '{}' returns an empty array.",
                    name,
                    struct_name
                );
            }
            let tokens = make_method_tokens(&method);
            acc.extend(tokens);
            acc
        })
}

fn make_method_tokens(props: &GenArray) -> proc_macro2::TokenStream {
    let field_idents = &props.fields;
    let count = field_idents.len();
    let return_type = &props.fn_ty;
    let vis = &props.vis;
    let fn_name = &props.fn_name;
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
    let field_idents = field_idents
        .iter()
        .map(|iae| {
            let ident = iae.ident.clone();
            match iae.cast {
                Some(CastKind::SafeCast) => quote! { #refa self.#ident as #return_type },
                Some(CastKind::UnsafeTransmute) => {
                    let source_ty = &iae.ty;
                    let refb = match source_ty {
                        Type::Reference(_) if props.is_ref => quote! {},
                        _ => quote! { #refa }
                    };
                    quote ! { unsafe { std::mem::transmute::<#refb #source_ty, #return_type>(#refa self.#ident) } }
                },
                None => quote! { #refa self.#ident }
            }
        });

    quote! {
        #[inline(always)]
        #vis fn #fn_name (& #muta self) -> [#return_type; #count] {
            [#(#field_idents),*]
        }
    }
}
