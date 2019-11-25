# Arraygen

[![Crates.io](https://img.shields.io/crates/v/arraygen.svg)](https://crates.io/crates/arraygen)
[![Docs](https://docs.rs/arraygen/badge.svg)](https://docs.rs/arraygen)

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

As you can see above, the `gen_array` attribute generates a new method returning an array of the indicated type. And the `in_array` attribute tells which fields are contained within the array returned by that method.

What `Arraygen` does under the hood is simply generating the following impl:

```rust
impl Person {
    #[inline(always)]
    fn get_names(&mut self) -> [&mut String; 2] {
        [&mut self.first_name, &mut self.last_name]
    }
}
```

#### The attribute `gen_array`

For generating an `Arraygen` method you have to use the attribute `gen_array` on top of the struct, indicating the method name and the return type.

```rust
#[derive(Arraygen)]
#[gen_array(fn get_strings: &String]
struct Foo {...}
```

In the code above, the struct `Foo` would have a new method with the following signature:

```rust
fn get_strings(&self) -> [&Strings; ?] {...}
```

#### The attribute `in_array`

In order to fill your `Arraygen` methods with struct fields, you have to use the attribute `in_array` in each struct field you want to include.

```rust
// inside a struct
#[in_array(get_strings)]
name: String,

#[in_array(get_strings)]
id: String,
```

You have to match the method name used in `gen_array` and in `in_array` in order to include those fields in the generated method.

## Documentation

For more information, check the [documentation page](https://docs.rs/arraygen).

## Limitations

There are not particular limitations, really. You can use this derive to return Copy objects, Trait objects, and basically any kind of object that can be a struct member.

Also, notice that there are no dynamic memory allocations involved.

The only drawback would be a little impact in compilation times.

## About the Syntax

I'm open to change the syntax for the 1.0 version. Participate in the issue [Syntax Proposals](https://github.com/theypsilon/arraygen/issues/1) to give your opinion on this matter.

## GettersByType

This crate is heavily inspired by [GettersByType](https://github.com/theypsilon/getters-by-type-rs) which is another derive that allows you
to do the same thing. But that one is more opinionated, less flexible and less powerful, with the only advantage of being less verbose. That's
why I decided to work on this new solution.

## License

MIT
