# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

## Version 0.3 - 2021-06-13

### Added
- Type wildcards for `implicit_select_all` clause. You now can use the wildcard `_` to match all types in its context.
- Decorators `cast` and `unsafe_transmute` that can be used on `in_array` attributes and on the `implicit_select_all` clause, for coverting field types to other types.

## Version 0.2 - 2021-05-09

### Added
- `implicit_select_all` clause for the `gen_array` attribute. With it you can select fields by type, without having to use `in_array`.

## Version 0.1 - 2019-11-24

### Added
- `gen_array`, and `in_array` attributes.