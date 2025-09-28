# What is this?

[![Crates.io](https://img.shields.io/crates/v/easy-macros-helpers-macro-safe.svg)](https://crates.io/crates/easy-macros-helpers-macro-safe)
[![Documentation](https://docs.rs/easy-macros-helpers-macro-safe/badge.svg)](https://docs.rs/easy-macros-helpers-macro-safe)

Easy Macros Helpers (Macro Safe) is a collection of utility functions provided and used by the [Easy Macros crate](https://crates.io/crates/easy_macros) that avoids dependency loops
by not depending on any other procedural macros from other crates included in the [Easy Macros](https://crates.io/crates/easy_macros).

## Core Features

### Error Context Generation

- [`context!`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/macro.context.html) - Generate context strings for error handling with automatic file/line information

### Token Stream Management

- [`TokensBuilder`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/struct.TokensBuilder.html) - Accumulate and combine token streams with methods inside
- [`readable_token_stream`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.readable_token_stream.html) - Format token strings for better readability
- [`token_stream_to_consistent_string`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.token_stream_to_consistent_string.html) - Normalize token representation across contexts

### Error Handling

- [`parse_macro_input!`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/macro.parse_macro_input.html) - Enhanced version of syn's macro that returns `Ok(TokenStream)` on parse errors (instead of `TokenStream`)
- [`expr_error_wrap`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.expr_error_wrap.html) with [`CompileErrorProvider`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/trait.CompileErrorProvider.html) trait - Wrap expressions with compile-time error reporting

### Code Generation Utilities

- [`indexed_name`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.indexed_name.html) - Generate indexed identifiers (`field0`, `field1`, etc.)
- [`find_crate`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.find_crate.html) - Locate crate references for generated code (supports renaming)
- [`find_crate_list`](https://docs.rs/easy-macros-helpers-macro-safe/latest/easy_macros_helpers_macro_safe/fn.find_crate_list.html) - Try multiple crates, return first found

## Examples

### Using `context!` for Error Handling

```rust,ignore
use std::fs;

fn load_config() -> anyhow::Result<String> {
    // Context with a custom message - this will be the failing operation
    fs::read_to_string("settings.json")
        .with_context(context!("Failed to load application settings"))
}

let result = load_config();
assert!(result.is_err());

let error_msg = format!("{:?}", result.unwrap_err());
assert!(error_msg.contains("Failed to load application settings"));
assert!(error_msg.contains("examples.rs"));
```

### Using `TokensBuilder` for Token Accumulation

```rust,ignore
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
assert_eq!(
    readable_token_stream(&tokens.to_string()),
    "{ println!(\"Hello\"); println!(\"World\"); }"
);
```

### Support for `Result<TokenStream, ...>` with `parse_macro_input!`

```rust,ignore
#[proc_macro]
#[anyhow_result]
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

```rust,ignore
let field_names = indexed_name(syn::parse_quote!(field), 3);
let output = quote! {
    struct MyStruct {
        #(#field_names: i32,)*
    }
};
assert_eq!(
    readable_token_stream(&output.to_string()),
    "struct MyStruct { field0: i32, field1: i32, field2: i32, }"
);
```

### Error Wrapping for Better Diagnostics

```rust,ignore
let mut errors = Vec::<String>::new();
let mut expr = syn::parse_quote!(some_expression);

// Add some errors
errors.push("This field is required".to_string());
errors.push("Invalid type specified".to_string());

// Wrap expression with compile errors
expr_error_wrap(&mut expr, &mut errors);
assert_eq!(
    quote! { #expr }.to_string(),
    quote! {
        {
            compile_error!("This field is required");
            compile_error!("Invalid type specified");
            some_expression
        }
    }
    .to_string()
);
```

### Finding Crate References (with Renaming Support)

```rust,ignore
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