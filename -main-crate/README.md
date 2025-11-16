# Easy Macros

[![Crates.io](https://img.shields.io/crates/v/easy-macros.svg)](https://crates.io/crates/easy-macros)
[![Documentation](https://docs.rs/easy-macros/badge.svg)](https://docs.rs/easy-macros)
[![License](https://img.shields.io/crates/l/easy-macros.svg)](https://github.com/LimitLost/easy-macros/blob/master/LICENSE)

Automatic error context for any Rust project + powerful procedural macro utilities.

## Quick Start

```toml
# Only automatic error context
[dependencies]
easy-macros = { version = "...", features = ["general"] }

# All tools for proc-macro development
[dependencies]
easy-macros = { version = "...", features = ["full"] }
```

## Features

### 1. Automatic Error Context - **Works in Any Project**

**Feature flag**: `always-context` (included in `general` and `full`)

Add `.with_context()` to all `?` operators automatically:

```rust
use easy_macros::always_context;
use anyhow::Result;

#[always_context]
fn process_data(user_id: u64) -> Result<Data> {
    let user = find_user(user_id)?;        // Auto-context with user_id
    let profile = load_profile(&user)?;     // Auto-context
    let data = fetch_data(profile.id)?;     // Auto-context
    Ok(data)
}
```

**Control attributes**: `#[no_context]`, `#[context(display)]`, `#[context(ignore)]`

### 2. Attribute Pattern Matching

**Feature flag**: `attributes` (included in `full`)

Extract values from attributes using `__unknown__` placeholder:

```rust
use easy_macros::{get_attributes, fields_get_attributes};

// Extract from type attributes
let routes: Vec<TokenStream> = get_attributes!(input, #[route(__unknown__)]);

// Extract from field attributes
let methods: Vec<(usize, Field, Vec<TokenStream>)> =
    fields_get_attributes!(input, #[http(__unknown__, "/api")]);

// Partial identifier matching
let tests: Vec<TokenStream> = get_attributes!(input, #[test_case___unknown__]);
// Matches: #[test_case_one], #[test_case_two], etc.
```

Check attributes: `has_attributes!(input, #[derive(Debug)] #[serde(...)])`  
Filter fields: `fields_with_attributes!(input, #[validate])`

### 3. Exhaustive AST Traversal

**Feature flag**: `all-syntax-cases` (included in `full`)

Generate recursive handlers for all syn types:

```rust
use easy_macros::all_syntax_cases;

all_syntax_cases! {
    setup => {
        generated_fn_prefix: "process",
        additional_input_type: &mut Context,
    }
    default_cases => {
        // Called for matching types across entire AST
        fn handle_expr(expr: &mut syn::Expr, ctx: &mut Context);

        #[after_system]  // Run after children processed
        fn finalize(item: &mut syn::Item, ctx: &mut Context);

        // Handle multiple syn types together (e.g., attributes + generics)
        fn check_attrs_and_generics(
            attrs: &mut Vec<syn::Attribute>,
            generics: &mut syn::Generics,
            ctx: &mut Context
        );
    }
    special_cases => {
        // Override for specific variants
        fn handle_call(call: &mut syn::ExprCall, ctx: &mut Context);
    }
}
```

Smart unwrapping of `Box<T>`, `Vec<T>`, `Punctuated<T, _>`. Generates handlers for Item, Expr, Stmt, Pat, Type, and more.

### 4. Helper Utilities

**Feature flags**: Individual helpers or `full` for all

```rust
use easy_macros::*;

// Manual error context with file/line info
// Feature: `context` (included in `general` and `full`)
fs::read("file.txt").with_context(context!("Loading config"))?;

// Token stream builder
// Feature: `tokens-builder` (included in `full`)
let mut tokens = TokensBuilder::default();
tokens.add(quote! { println!("Hello"); });
tokens.braced();  // Wrap in { }

// Parse with compile_error! on failure
// Feature: `parse-macro-input` (included in `full`)
let parsed = parse_macro_input!(input as syn::DeriveInput);

// Generate indexed names: field0, field1, field2
// Feature: `indexed-name` (included in `full`)
let names = indexed_name(syn::parse_quote!(field), 3);

// Find crates (handles renames)
// Feature: `find-crate` (included in `full`)
let path = find_crate("serde", quote!(::Serialize))?;
let async_rt = find_crate_list(&[("tokio", quote!()), ("async-std", quote!())])?;
```

### 5. Result Type for Proc Macros

**Feature flag**: `anyhow-result` (included in `full`)

Use `anyhow::Result<TokenStream>` in proc macros:

```rust
use easy_macros::anyhow_result;

#[proc_macro_derive(MyTrait)]
#[anyhow_result]
fn derive_my_trait(input: TokenStream) -> anyhow::Result<TokenStream> {
    let parsed: syn::DeriveInput = syn::parse(input)?;
    anyhow::ensure!(!parsed.fields.is_empty(), "Struct must have fields");
    Ok(quote! { /* generated */ }.into())
}
// Errors convert to compile_error! automatically
```

## Complete Example

```rust
use easy_macros::{
    anyhow_result, always_context, fields_get_attributes, TokensBuilder, parse_macro_input
};

#[proc_macro_derive(Routes, attributes(route))]
#[anyhow_result]
#[always_context]
pub fn derive_routes(input: TokenStream) -> anyhow::Result<TokenStream> {
    let input = parse_macro_input!(input as DeriveInput);

    let routes: Vec<(usize, Field, Vec<TokenStream>)> =
        fields_get_attributes!(input, #[route(__unknown__)]);

    anyhow::ensure!(!routes.is_empty(), "No routes found");

    let mut output = TokensBuilder::default();
    for (_, field, methods) in routes {
        let name = field.ident.as_ref().context("Field needs name")?;
        for method in methods {
            output.add(quote! {
                pub fn #name() -> Route { Route::new(stringify!(#method)) }
            });
        }
    }

    Ok(quote! { impl Routes { #output } }.into())
}
```

## Feature Flags

### Feature Groups

- **`general`** - Automatic error context for any project

  - Includes: `always-context`, `context`
  - Use when you only need automatic error context, not proc-macro development tools

- **`full`** - Complete toolkit for proc-macro development

  - Includes: `all-syntax-cases`, `always-context`, `attributes`, `anyhow-result`, and all helpers
  - Use when building procedural macros or need the full feature set

- **`build`** - Build-time macro that auto-adds `#[always_context]` to all functions returning `anyhow::Result`
  - Standalone feature, not included in `general` or `full`
  - Add to `[build-dependencies]` and configure via `build.rs`

### Individual Features

**Core Proc-Macro Tools**:

- **`all-syntax-cases`** - Exhaustive AST traversal and handler generation
- **`always-context`** - `#[always_context]` attribute for automatic error context
- **`attributes`** - Attribute pattern matching macros (`has_attributes!`, `get_attributes!`, etc.)
- **`anyhow-result`** - `#[anyhow_result]` for using `anyhow::Result<TokenStream>` in proc-macros

**Helper Utilities** (granular control):

- **`context`** - `context!()` macro for manual error context with file/line info
- **`tokens-builder`** - `TokensBuilder` for incrementally building token streams
- **`indexed-name`** - `indexed_name()` for generating indexed identifiers
- **`find-crate`** - `find_crate()` and `find_crate_list()` for locating crates with rename support
- **`parse-macro-input`** - `parse_macro_input!()` with automatic `compile_error!` on parse failure
- **`expr-error-wrap`** - `expr_error_wrap()` utilities for wrapping expressions
- **`readable-token-stream`** - Token stream formatting utilities
- **`token-stream-consistent`** - Consistent token stream string conversion

### Integration Features

- **`easy-sql`** - Integration with Easy-SQL crate
  - Adds Easy-SQL support to `#[always_context]` and attribute macros
  - Optional: only needed if using Easy-SQL in your project

## License

Apache License, Version 2.0 - See [LICENSE](LICENSE)
