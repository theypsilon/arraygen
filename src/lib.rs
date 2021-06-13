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
//! let mut person = Person {
//!     first_name: "Ada".into(),
//!     last_name: "Lovelace".into()
//! };
//!
//! for name in person.get_names().iter_mut() {
//!     **name = name.to_lowercase();
//! }
//!
//! assert_eq!(
//!     format!("{:?}", person),
//!     "Person { first_name: \"ada\", last_name: \"lovelace\" }"
//! );
//! ```
//!
//! As you can see above, the attribute `gen_array` generates a new method returning an array of the given type.
//! And then, the attribute `in_array` indicates the fields to be included by that method. In this case, the
//! generated method 'get_names' will return an array including references to all the fields of the struct.
//!
//! As you might have guessed, what `Arraygen` does under the hood is simply generating the following impl:
//!
//! ```rust
//! # struct Person {
//! #     first_name: String,
//! #     last_name: String,
//! # }
//! impl Person {
//!     #[inline(always)]
//!     fn get_names(&mut self) -> [&mut String; 2] {
//!         [&mut self.first_name, &mut self.last_name]
//!     }
//! }
//! ```

#![allow(clippy::eval_order_dependence)]

extern crate proc_macro;

use proc_macro::TokenStream;

const DERIVE_NAME: &str = "Arraygen";
const DECL_FN_NAME: &str = "gen_array";
const FIELD_SELECTOR_NAME: &str = "in_array";
const IMPLICIT_SELECT_ALL_NAME: &str = "implicit_select_all";

/// The `Arraygen` derive allows you to use the attribute `gen_array` at the struct level, and the attribute `in_array` in each contained field.
///
///
/// # gen_array
///
/// With `gen_array` you can declare your `Arraygen` methods in the following way:
///
/// ```ignore
/// #[gen_array(?visibility fn your_method_name: YourReturnType)]
/// ```
///
/// * **?visibility**: This placeholder is optional. You can let it blank entirely. Or you can write `pub`, `pub(crate)` or any other pub variant.
/// * **your_method_name**: This is meant to be any valid method name, following the standard rules. You can't use a name taken by another method in the struct impl. This restriction also includes other `Arraygen` methods.
/// * **YourReturnType**: The return type can be any Rust type that can appear in a struct field. Notice that if the `type` does not implement the trait `Copy`, you are better returning `&type` or `&mut type` instead, in order to avoid ownership errors.
///
/// There is no limit in the number of methods you can declare.
///
/// By default, these new `Arraygen` methods return arrays of length 0. That's not very useful, but that's why we also have the next attribute: `in_array`.
///
///
/// # in_array
///
/// With `in_array` you select which field is returned by which method generated by `gen_array`.
///
/// ```ignore
/// #[in_array(your_method_name)]
/// ```
///
/// * `your_method_name`: This needs to match the name of some method declared in the same struct by the `gen_array` attribute.
///
///
/// This is the way to fill up your `Arraygen` methods. The only thing you need to care about is that the type returned by `your_method_name` needs to be compatible with the type of the field with the `in_array` attribute.
///
/// Notice that in Rust, non-reference field types can be returned as references, but not the other way around. Or in other words. This is good:
///
/// ```rust
/// # use arraygen::Arraygen;
/// #[derive(Arraygen)]
/// #[gen_array(fn references: &i32)]
/// struct Test {
///     #[in_array(references)]
///     data: i32
/// }
/// ```
///
/// But this is bad:
///
/// ```compile_fail
/// # use arraygen::Arraygen;
/// #[derive(Arraygen)]
/// #[gen_array(fn non_references: i32)]
/// struct Test<'a> {
///     #[in_array(non_references)]
///     data: &'a i32
/// }
/// ```
///
/// Is also good to know that the same field can be included in many `Arraygen` methods, not just in only one.
/// You will see what I mean by checking the following example:
///
/// ```rust
/// # use arraygen::Arraygen;
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
///     #[in_array(odds, primes)] // This syntax is also valid, by the way.
///     three: i32,
///
///     #[in_array(evens)]
///     four: i32,
///
///     #[in_array(odds, primes)]
///     five: i32
/// }
///
/// let numbers = Numbers {
///     one: 1,
///     two: 2,
///     three: 3,
///     four: 4,
///     five: 5
/// };
///
/// assert_eq!(numbers.odds(), [1, 3, 5]);
/// assert_eq!(numbers.evens(), [2, 4]);
/// assert_eq!(numbers.primes(), [2, 3, 5]);
/// ```
///
/// You may also add decorators to your `in_array` attribute.
///
/// ```ignore
/// #[in_array(your_method_name { comma_separated_decorators })]
/// ```
///
/// Possible decorators are:
///
/// - cast : Casts the field type to the return type of the `gen_array` method.
/// - unsafe_transmute : Uses unsafe { std::mem::transmute } the type of the field to the return type of the `gen_array` method.
/// - override_implicit : In case the field is already selected by a `implicit_select_all` clause for this `gen_array` (more about this clause later), you may use `override_implicit` to apply different decorators to the current field.
///
/// Casting example:
///
/// ```rust
/// # use arraygen::Arraygen;
/// #[derive(Arraygen)]
/// #[gen_array(fn all: i32)]
/// struct Numbers {
///     #[in_array(all { cast })]
///     one: f32,
///
///     #[in_array(all { cast })]
///     two: u8,
///
///     #[in_array(all { cast })]
///     three: bool,
/// }
///
/// let numbers = Numbers {
///     one: 1.0,
///     two: 1,
///     three: true
/// };
///
/// assert_eq!(numbers.all(), [1, 1, 1]);
///
/// ```
///
///
///
/// # Trait Objects
///
/// A very good use-case for `Arraygen` would be being able to extract Trait Objects from different concrete types, so you can operate in all of them at once.
///
/// ```rust
/// # use arraygen::Arraygen;
/// trait Animal {
///     fn talk(&self) -> &'static str;
/// }
/// # struct Dog {}
/// # impl Animal for Dog {
/// #    fn talk(&self) -> &'static str {
/// #        "bark"
/// #     }
/// # }
/// # struct Cat {}
/// # impl Animal for Cat {
/// #    fn talk(&self) -> &'static str {
/// #        "meow"
/// #    }
/// # }
/// # struct Pig {}
/// # impl Animal for Pig {
/// #    fn talk(&self) -> &'static str {
/// #        "oink"
/// #    }
/// # }
///
/// #[derive(Arraygen)]
/// #[gen_array(fn get_animals: &dyn Animal)]
/// struct Animals {
///     #[in_array(get_animals)]
///     dogo: Dog,
///     #[in_array(get_animals)]
///     tiger: Cat,
///     #[in_array(get_animals)]
///     bacon: Pig,
/// }
///
/// let animals = Animals {
///     dogo: Dog {},
///     tiger: Cat {},
///     bacon: Pig {}
/// };
///
/// let talk: Vec<&'static str> = animals
///     .get_animals()
///     .iter()
///     .map(|animal| animal.talk())
///     .collect();
///
/// assert_eq!(talk, ["bark", "meow", "oink"]);
/// ```
///
/// And a more realistic example could be this other one:
///
/// ```
/// # use arraygen::Arraygen;
/// trait SetNone {
///     fn set_none(&mut self);
/// }
///
/// impl<T> SetNone for Option<T> {
///     fn set_none(&mut self) {
///         *self = None;
///     }
/// }
///
/// #[derive(Arraygen)]
/// #[gen_array(fn ephemeral_options: &mut dyn SetNone)]
/// struct ManyOptions {
///     #[in_array(ephemeral_options)]
///     a: Option<i32>,
///     #[in_array(ephemeral_options)]
///     b: Option<String>,
///     c: Option<String>,
/// }
///
/// let mut many = ManyOptions {
///     a: Some(42),
///     b: Some(String::from("foo")),
///     c: Some(String::from("bar"))
/// };
///
/// for option in many.ephemeral_options().iter_mut() {
///     option.set_none();
/// }
///
/// assert_eq!(many.a, None);
/// assert_eq!(many.b, None);
/// assert_eq!(many.c, Some(String::from("bar")));
/// ```
///
/// With ad-hoc traits and `Arraygen` is very easy to generalize common transformations with simple one-liners.
///
///
/// # Implicit selection of Fields by their Types
///
/// You may omit entirely the `in_array` attribute if you use the `implicit_select_all` clause in your `gen_array` declarations.
///
/// ```ignore
/// #[gen_array(?visibility fn your_method_name: YourReturnType, implicit_select_all: MatchingFieldTypes)]
/// ```
///
/// You may place either a single type in your `implicit_select_all` clause or a list of comma-separated types.
///
/// ```rust
/// # use arraygen::Arraygen;
/// #[derive(Arraygen)]
/// #[gen_array(fn get_all_prices: f32, implicit_select_all: f32)]
/// struct ImplicitPrices {
///     pub water: f32,
///     pub oil: f32,
///     pub tomato: f32,
///     pub chocolate: f32,
/// }
///
/// let prices = ImplicitPrices {
///     water: 2.0,
///     oil: 4.0,
///     tomato: 3.0,
///     chocolate: 5.0,
/// };
///
/// assert_eq!(prices.get_all_prices().iter().sum::<f32>(), 14.0);
/// ```
///
/// The `implicit_select_all` clause may also include decorators:
/// ```ignore
/// #[gen_array(?visibility fn your_method_name: YourReturnType, implicit_select_all { comma_separated_decorators }: MatchingFieldTypes)]
/// ```
///
/// See the about which decorators you may use in the previous `in_array` section.
///
/// # Implicit selection of Fields with Type Wilcards
///
/// You may use Type Wildcards (`_`) on the `implicit_select_all` clause.
///
/// For example, next expression will match all fields regardless of their type (this might be used in conjunction with decorators for casting between types):
///
/// ```ignore
/// #[gen_array(fn all_fields: f32, implicit_select_all: _)]
/// ```
///
/// Type Wildcards may be used within a more complex Type definition, like `Option < _ >` or `Result < f32, _ >`. Example:
///
/// ```rust
/// # use arraygen::Arraygen;
/// #[derive(Arraygen, Debug)]
/// #[gen_array(fn options: &mut dyn ResetOption, implicit_select_all: Option<_>)]
/// struct Options {
///     pub a: Option<i32>,
///     pub b: Option<bool>
/// }
///
/// impl<T> ResetOption for Option<T> {
///     fn reset(&mut self) {
///         *self = None;
///     }
/// }

/// trait ResetOption {
///     fn reset(&mut self);
/// }
///
/// let mut options = Options {
///     a: Some(1),
///     b: Some(true)
/// };
///
/// options.options().iter_mut().for_each(|o| o.reset());
/// assert_eq!(format!("{:?}", options), "Options { a: None, b: None }");
/// ```
///

#[proc_macro_derive(Arraygen, attributes(gen_array, in_array))]
pub fn arraygen(input: TokenStream) -> TokenStream {
    transform_context::transform_ast(input)
}

mod parse_attribute;
mod parse_decorator;
mod parse_derive_arraygen;
mod parse_gen_array;
mod parse_in_array;
mod transform_context;
mod types;
mod utils;
