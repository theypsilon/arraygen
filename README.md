# Arraygen

[![Crates.io](https://img.shields.io/crates/v/arraygen.svg)](https://crates.io/crates/arraygen)
[![Docs](https://docs.rs/arraygen/badge.svg)](https://docs.rs/arraygen)
[![Twitter](https://img.shields.io/twitter/url/https/twitter.com/josembarroso.svg?style=social&label=Follow%20%40josembarroso)](https://twitter.com/josembarroso)
<span class="badge-buymeacoffee"><a href="https://ko-fi.com/theypsilon" title="Buy Me a Coffee at ko-fi.com'"><img src="https://img.shields.io/badge/buy%20me%20a%20coffee-donate-yellow.svg" alt="Buy Me a Coffee at ko-fi.com'" /></a></span>

This crate provides `Arraygen` derive macro for structs, which generates methods returning arrays filled with the selected struct fields.

#### Complete example:

```rust
#[derive(Arraygen, Debug)]
#[gen_array(fn get_names: &mut String)]
struct Person {
    #[in_array(get_names)]
    first_name: String,
    #[in_array(get_names)]
    last_name: String,
}

let mut person = Person {
    first_name: "Ada".into(),
    last_name: "Lovelace".into()
};

for name in person.get_names().iter_mut() {
    **name = name.to_lowercase();
}

assert_eq!(
    format!("{:?}", person),
    "Person { first_name: \"ada\", last_name: \"lovelace\" }"
);
// PASSES !
// Notice how it was not lowercased on object creation
// but now it is.
```

As you can see above, the `gen_array` attribute generates a new method returning an array of the indicated type. Additionally, the `in_array` attribute tells which fields are contained within the array returned by that method.

What `Arraygen` does under the hood is simply generating the following impl:

```rust
impl Person {
    #[inline(always)]
    fn get_names(&mut self) -> [&mut String; 2] {
        [&mut self.first_name, &mut self.last_name]
    }
}
```

#### The attribute `gen_array`:

For generating an `Arraygen` method you have to use the attribute `gen_array` on top of the struct. There you will indicate the method name and the return type. Optionally, you can also indicate the visibility or an `implicit_select_all` clause. In the following example you'll see how to tweak the visibility:

```rust
#[derive(Arraygen)]
#[gen_array(pub(crate) fn get_strings: &String)]
struct Foo {...}
```

In the code above, the struct `Foo` would have a new method with the following signature:

```rust
pub(crate) fn get_strings(&self) -> [&String; ?] {...}
```

#### The attribute `in_array`:

To fill your `Arraygen` methods with struct fields, you have to use the attribute `in_array` in each struct field you want to include.

```rust
// inside a struct
#[in_array(get_strings)]
id: String,

#[in_array(get_strings, get_names)]
name: String,
```

You have to match the method name used in `gen_array` and `in_array` to include these fields in each generated method. So in this example, assuming `gen_strings` and `get_names` are both generated by `gen_array`, the former will get populated with the fields `id` and `name`, and the latter will get populated with the field `name`.

It is also possible to entirely omit the attribute `in_array` with the use of an `implicit_select_all` clause. Check the ["implicit_select_all" section in the documentation](https://docs.rs/arraygen/0.3.0/arraygen/derive.Arraygen.html#implicitly-selection-fields-by-their-types) to learn more about this possibility.


#### Generating arrays of Trait Objects:

Trait Objects are fully supported, check the [Trait Objects section in the documentation](https://docs.rs/arraygen/0.3.0/arraygen/derive.Arraygen.html#trait-objects) to see a few working examples.

#### Implicit selection of Fields by their Types

With the clause `implicit_select_all`, you may select fields without using `in_array`, check [this section in the documentation](https://docs.rs/arraygen/0.3.0/arraygen/derive.Arraygen.html#implicitly-selection-fields-by-their-types) to see an example.


## Documentation

For more information, check the [documentation page](https://docs.rs/arraygen).

## Usage

With Cargo, you can add this line to your Cargo.toml:

```toml
[dependencies]
arraygen = "0.3.0"
```

## About the Syntax

I'm open to change the syntax for the 1.0 version. Participate in the issue [Syntax Proposals](https://github.com/theypsilon/arraygen/issues/1) to give your opinion on this matter.

## Known Problems

Error messages could be improved in a few cases. And there is no proper support for warnings yet. Arraygen prints standard messages instead of warnings at the moment.

## GettersByType

This crate is heavily inspired by [GettersByType](https://github.com/theypsilon/getters-by-type-rs) which is another derive that allows you
to do the same thing. But that one is more opinionated, less flexible, and less powerful, with the only advantage of being less verbose. That's
why I decided to work on this new solution.

## License

MIT
