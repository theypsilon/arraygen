use quote::quote;
use regex::Regex;
use syn::Type;

pub fn are_matching_types(left_ty: &Type, right_ty: &Type) -> bool {
    ty_inferred_by(left_ty, right_ty) || ty_inferred_by(right_ty, left_ty)
}

pub fn ty_inferred_by(field_ty: &Type, implicit_ty: &Type) -> bool {
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

#[allow(non_snake_case)]
#[cfg(test)]
mod test {
    use super::*;

    macro_rules! ty_inferred_by_tests {
        ($($name:ident: $str1:expr, $str2:expr, $expected:expr)*) => {
        $(
            #[test]
            fn $name() {
                assert_eq!(
                    ty_inferred_by(
                        &syn::parse_str($str1).unwrap(),
                        &syn::parse_str($str2).unwrap()
                    ),
                    $expected
                );
            }
        )*
        }
    }

    ty_inferred_by_tests! {
        ty_inferred_by___with_exactly_same_option___returns_true: "Option<i32>", "Option<i32>", true
        ty_inferred_by___with_exactly_same_complex_ty___returns_true: "Option<Result<(i32, f32), std::fmt::Error>>", "Option<Result<(i32, f32), std::fmt::Error>>", true
        ty_inferred_by___with_different_result1___returns_false: "Option<Result<(i32, f32), std::fmt::Error>>", "Option<Result<(i32, i32), std::fmt::Error>>", false
        ty_inferred_by___with_different_result2___returns_false: "Option<Result<(i32, f32), std::error::Error>>", "Option<Result<(i32, f32), std::fmt::Error>>", false
        ty_inferred_by___compared_to_wildcard1___returns_true: "Option<i32>", "_", true
        ty_inferred_by___compared_to_wildcard2___returns_true: "Option<i32>", "Option<_>", true
        ty_inferred_by___compared_to_wildcard3___returns_true: "Option<Option<i32>>", "Option<Option<_>>", true
        ty_inferred_by___compared_to_wildcard4___returns_true: "Option<Result<i32, i32>>", "Option<Result<_, i32>>", true
        ty_inferred_by___compared_to_wildcard5___returns_false: "Option<Result<i32, i32>>", "Option<Result<_, f32>>", false
    }
}
