# Easy Macros

[![Crates.io](https://img.shields.io/crates/v/easy-macros.svg)](https://crates.io/crates/easy-macros)
[![Documentation](https://docs.rs/easy-macros/badge.svg)](https://docs.rs/easy-macros)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

Use the [parent crate](https://crates.io/crates/easy_macros) instead.

## What is this?

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

```rust
use easy_macros::TokensBuilder;
use quote::quote;

let mut result = TokensBuilder::default();

// Add multiple token streams
result.add(quote! {
    println!("Hello");
});
result.add(quote! {
    println!("World");
});

// Wrap everything in braces
result.braced();

// Get final result
let tokens = result.finalize();
// Result: { println!("Hello"); println!("World"); }
```

### Support for `Result<TokenStream, ...>` with `parse_macro_input!`

```rust
use easy_macros::parse_macro_input;
use proc_macro::TokenStream;

#[proc_macro]
pub fn my_macro(input: TokenStream) -> anyhow::Result<TokenStream> {
    //This doesn't return TokenStream on compile errors, but Ok(TokenStream) with compile_error! inside
    let parsed = parse_macro_input!(input as syn::DeriveInput);

    // Process parsed input...
    Ok(quote! {
        // Generated code
    }.into())
}
```

### Generating Indexed Names

```rust
use easy_macros::indexed_name;
use quote::quote;

let field_names = indexed_name(syn::parse_quote!(field), 3);
let output = quote! {
    struct MyStruct {
        #(#field_names: i32,)*
    }
};
// Generates: struct MyStruct { field0: i32, field1: i32, field2: i32, }
```

### Error Wrapping for Better Diagnostics

```rust
use easy_macros::{expr_error_wrap, ErrorData};

let mut errors = Vec::<String>::new();
let mut expr = syn::parse_quote!(some_expression);

// Add some errors
errors.push("This field is required".to_string());
errors.push("Invalid type specified".to_string());

// Wrap expression with compile errors
expr_error_wrap(&mut expr, &mut errors);
// The expression now includes compile_error! calls
```

### Finding Crate References (with Renaming Support)

```rust
use easy_macros::{find_crate, find_crate_list};
use quote::quote;

// Simple crate lookup
if let Some(path) = find_crate("serde", quote!(::Serialize)) {
    // Use path in generated code
}

// Handles renamed crates automatically
// If Cargo.toml has: serde_renamed = { package = "serde", version = "1.0" }
// The above will return: serde_renamed::Serialize

// Try multiple crates with fallbacks
let async_crates = &[
    ("tokio", quote!(::runtime::Runtime)),
    ("async-std", quote!(::task)),
    ("smol", quote!()),
];

if let Some(async_path) = find_crate_list(async_crates) {
    // Uses first available async runtime
}
```
