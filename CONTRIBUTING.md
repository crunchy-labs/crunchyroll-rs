# Contributing

The following sections should give you a pretty good overview what to do if you plan to contribute to this library and how to do it.
For this, please especially respect our [conventions](#conventions) since some of them are critical for testing.

## Table of contents
- [Conventions](#conventions)
  - [Git](#git)
  - [Struct](#struct)
- [Testing](#testing)

## Conventions

Some conventions must be respected in order to guarantee correct test execution / results and consistent practises.
Please follow the rules carefully.

### Git

Our commit messages are following a simple scheme: Present tense and a (first letter uppercase) keyword at the beginning which represents the change(s) a commit does.
`Add feature ...`, `Fix ... (#69)`, `Remove deprectaed field from ...`, ... .
You can always look at the [commit history](https://github.com/crunchy-labs/crunchyroll-rs/commits) to see all commit message if you're unsure how to write it.

### Struct

_**This section is only necessary for structs which get used to scan crunchyroll api results in.**_

When defining a new struct two attributes must be specified for it.
`#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]` is required for testing.
It ensures that the api result does not contain any unknown fields when running the tests with the `__test_strict` feature.
`#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]` enables default values for all fields.
If the api changes at some point and some fields are getting removed, the library does not fail immediately and uses the default value for the field instead.
Some field attributes might not impl `Default` and therefore `derive(Default)` causes a compile error.
For this case, the [`smart-default`](https://github.com/idanarye/rust-smart-default) crate is a dependency.
Use `derive(SmartDefault)` instead of `derive(Default)` and then set the default value for the field manually via `smart-default`.
See [here](https://github.com/crunchy-labs/crunchyroll-rs/blob/3509cdd6d4d3e92ee98e7ecaea27f36c07c71914/src/crunchyroll.rs#L254) how it's used in practice (since [`chrono::DateTime`](https://github.com/chronotope/chrono) does not support / impl `Default`).
This attribute gets disabled when the library is tested with the `__test_strict` feature to ensure failing when a field is missing.

Some api results are "polluted" with fields which are not really necessary.
To guarantee test integrity with `__test_strict` these fields must also be included.
Attribute the fields with `#[cfg(feature = "__test_strict")]` to only include them when testing with the `__test_strict` feature.
This ensures that no extra memory is allocated for field values which will never be used.
Use `StrictValue` as type for the fields.

A struct after the named conventions looks like this:
```rust
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]
pub struct ExampleResponse {
    ...
    
    #[cfg(feature = "__test_strict")]
    __object__: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    __owo__: crate::StrictValue,
    
    ...
}
```

## Testing

Before submitting new code make sure to run it against our test suite to check if the library works as intended.
If you implement code which does something completely new (like new api endpoints) and does not get used by any existing test, write tests for it which covers your new code completely.
When changing the library in a way that it is incompatible with prior version note these changes in your PR (or however the changes are published) to find the source of these changes easier.

All test are in the [tests](tests) directory and can be run with the following two commands:
```shell
$ cargo test
$ cargo test --features __test_strict
```
