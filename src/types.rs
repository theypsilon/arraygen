use proc_macro2::{token_stream::IntoIter, TokenTree, TokenTree::*};
use quote::quote;
use syn::Type;

pub fn are_matching_types(left_ty: &Type, right_ty: &Type) -> bool {
    compare_types(left_ty, right_ty, true)
}

pub fn ty_inferred_by(field_ty: &Type, implicit_ty: &Type) -> bool {
    compare_types(field_ty, implicit_ty, false)
}

fn compare_types(left_ty: &Type, right_ty: &Type, wildcards_on_left: bool) -> bool {
    if *left_ty == *right_ty {
        return true;
    }
    if let Type::Infer(_) = right_ty {
        return true;
    }
    if wildcards_on_left {
        if let Type::Infer(_) = left_ty {
            return true;
        }
    }

    let mut right_tokens = quote! { #right_ty }.into_iter();
    let mut left_tokens = quote! { #left_ty }.into_iter();

    let mut last_group = 'Z';

    loop {
        let (left_t, right_t) = match (left_tokens.next(), right_tokens.next()) {
            (Some(lt), Some(rt)) => (lt, rt),
            (Some(_), None) | (None, Some(_))=> return false,
            (None, None) => return true
        };

        match right_t {
            Punct(ref p) if p.as_char() == '(' || p.as_char() == '<' || p.as_char() == '[' => {
                last_group = p.as_char()
            }
            _ => {}
        }

        match (&left_t, &right_t) {
            (Punct(p1), Punct(p2)) if p1.as_char() == p2.as_char() => continue,
            (Ident(i1), Ident(i2)) if i1 == i2 => continue,
            (Literal(l1), Literal(l2)) if l1.to_string() == l2.to_string() => continue,
            (Group(g1), Group(g2)) if g1.to_string() == g2.to_string() => continue,
            _ => {}
        }

        let mut termination = AdvanceTermination { wildcard_ended: false, other_ended: false };

        if advance_if_wildcard(&right_t, &mut right_tokens, &mut left_tokens, last_group, &mut termination)
            || (wildcards_on_left
                && advance_if_wildcard(&left_t, &mut left_tokens, &mut right_tokens, last_group, &mut termination))
        {
            match (termination.wildcard_ended, termination.other_ended) {
                (true, true) => return true,
                (true, false) | (false, true) => return false,
                (false, false) => continue
            }
        }
        return false;
    }
}

struct AdvanceTermination {
    pub wildcard_ended: bool,
    pub other_ended: bool
}

fn advance_if_wildcard(
    wildcard_token: &TokenTree,
    wildcard_iter: &mut IntoIter,
    other_iter: &mut IntoIter,
    last_group: char,
    termination: &mut AdvanceTermination
) -> bool {
    if !matches!(wildcard_token, Ident(ref p) if p == "_") {
        return false;
    }

    match wildcard_iter.next() {
        Some(_) => {},
        None => {
            termination.wildcard_ended = true;
        }
    }

    for other_token in other_iter {
        match other_token {
            Punct(ref p)
                if (p.as_char() == ')' && last_group == '(')
                    || (p.as_char() == '>' && last_group == '<')
                    || (p.as_char() == ']' && last_group == '[')
                    || p.as_char() == ',' =>
            {
                return true;
            }
            _ => {}
        }
    }
    termination.other_ended = true;
    true
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
        ty_inferred_by___with_slightly_different_complex_ty_1___returns_false: "Option<Result<(i32, f32), std::fmt::Error>>", "Option<Result<(i32, i32), std::fmt::Error>>", false
        ty_inferred_by___with_slightly_different_complex_ty_2___returns_false: "Option<Result<(i32, f32), std::error::Error>>", "Option<Result<(i32, f32), std::fmt::Error>>", false
        ty_inferred_by___compared_to_wildcard_1___returns_true: "Option<i32>", "_", true
        ty_inferred_by___compared_to_wildcard_2___returns_true: "Option<i32>", "Option<_>", true
        ty_inferred_by___compared_to_wildcard_3___returns_true: "Option<Option<i32>>", "Option<Option<_>>", true
        ty_inferred_by___compared_to_wildcard_4___returns_true: "Option<Result<i32, i32>>", "Option<Result<_, i32>>", true
        ty_inferred_by___compared_to_wildcard_5___returns_false: "Option<Result<i32, i32>>", "Option<Result<_, f32>>", false
        ty_inferred_by___compared_to_wildcard_6___returns_true: "Result<i32, Option<i32>>", "Result<_, Option<i32>>", true
        ty_inferred_by___with_matching_wildcards_in_both_sides_1___returns_true: "Result<_, i32>", "Result<_, i32>", true
        ty_inferred_by___with_matching_wildcards_in_both_sides_2___returns_true: "Result<i32, _>", "Result<i32, _>", true
    }

    macro_rules! are_matching_types_tests {
        ($($name:ident: $str1:expr, $str2:expr, $expected:expr)*) => {
        $(
            #[test]
            fn $name() {
                assert_eq!(
                    are_matching_types(
                        &syn::parse_str($str1).unwrap(),
                        &syn::parse_str($str2).unwrap()
                    ),
                    $expected
                );
            }
        )*
        }
    }

    are_matching_types_tests! {
        are_matching_types___with_matching_wildcards_in_both_sides___returns_true: "Result<i32, _>", "Result<_, i32>", true
        are_matching_types___between_wildcard_and_any_other_type___returns_true: "_", "Option<f32>", true
    }
}
