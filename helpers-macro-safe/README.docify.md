# What is this?

[![Crates.io](https://img.shields.io/crates/v/easy-macros-helpers-macro-safe.svg)](https://crates.io/crates/easy-macros-helpers-macro-safe)
[![Documentation](https://docs.rs/easy-macros-helpers-macro-safe/badge.svg)](https://docs.rs/easy-macros-helpers-macro-safe)

Easy Macros Helpers (Macro Safe) is a collection of utility functions provided and used by the [Easy Macros crate](https://crates.io/crates/easy_macros) that avoids dependency loops
by not depending on any other procedural macros from other crates included in the [Easy Macros](https://crates.io/crates/easy_macros).

## Core Features

### Token Stream Management

- [`TokensBuilder`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/struct.TokensBuilder.html) - Accumulate and combine token streams with methods inside
- [`readable_token_stream`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.readable_token_stream.html) - Format token strings for better readability
- [`token_stream_to_consistent_string`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.token_stream_to_consistent_string.html) - Normalize token representation across contexts

### Error Handling

- [`parse_macro_input!`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/macro.parse_macro_input.html) - Enhanced version of syn's macro that returns `Ok(TokenStream)` on parse errors (instead of `TokenStream`)
- [`expr_error_wrap`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.expr_error_wrap.html) with [`ErrorData`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/trait.ErrorData.html) trait - Wrap expressions with compile-time error reporting

### Code Generation Utilities

- [`indexed_name`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.indexed_name.html) - Generate indexed identifiers (`field0`, `field1`, etc.)
- [`find_crate`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.find_crate.html) - Locate crate references for generated code (supports renaming)
- [`find_crate_list`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.find_crate_list.html) - Try multiple crates, return first found

## Examples

### Using `TokensBuilder` for Token Accumulation

<!-- docify::embed!("src/examples.rs", readme_tokens_builder_example) -->

### Support for `Result<TokenStream, ...>` with `parse_macro_input!`

```rust,ignore
#[proc_macro]
#[macro_result]
fn my_macro(input: TokenStream) -> anyhow::Result<TokenStream> {
    //This doesn't return TokenStream on compile errors, but Ok(TokenStream) with compile_error! inside
    let parsed = parse_macro_input!(input as syn::DeriveInput);

    // Process parsed input...
    Ok(quote::quote! {
        // Generated code
    }.into())
}
```

### Generating Indexed Names

<!-- docify::embed!("src/examples.rs", readme_indexed_name_example) -->

### Error Wrapping for Better Diagnostics

<!-- docify::embed!("src/examples.rs", readme_error_wrapping_example) -->

### Finding Crate References (with Renaming Support)

<!-- docify::embed!("src/examples.rs", readme_find_crate_example) -->
