# What is this?

[![Crates.io](https://img.shields.io/crates/v/easy-macros-add-code.svg)](https://crates.io/crates/easy-macros-add-code)

`#[add_code]` injects statements at the start and/or end of a function or impl method body.
Its main use case is keeping docify-generated examples clean while still adding setup/teardown or assertions for tests.
