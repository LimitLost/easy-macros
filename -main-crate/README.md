# Easy Macros

[![Crates.io](https://img.shields.io/crates/v/easy-macros.svg)](https://crates.io/crates/easy-macros)
[![Documentation](https://docs.rs/easy-macros/badge.svg)](https://docs.rs/easy-macros)
[![License](https://img.shields.io/crates/l/easy-macros.svg)](https://github.com/LimitLost/easy-macros/blob/master/LICENSE)

Automatic error context for any Rust project + powerful procedural macro utilities.

## Table of Contents

- [Quick Start](#quick-start)
- [Features](#features)
  - [1. Automatic Error Context - **Works in Any Project**](#1-automatic-error-context---works-in-any-project)
  - [2. Attribute Pattern Matching](#2-attribute-pattern-matching)
  - [3. Exhaustive AST Traversal](#3-exhaustive-ast-traversal)
  - [4. Helper Utilities](#4-helper-utilities)
  - [5. Result Type for Proc Macros](#5-result-type-for-proc-macros)
  - [6. Add Code - Make (Docify) Examples minimal](#6-add-code---make-docify-examples-minimal)
- [Feature Flags](#feature-flags)

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

```rust,ignore
let user_id = 123u64;
let user = find_user(user_id)?; // Auto-context with user_id
let profile = load_profile(&user)?; // Auto-context
let data = fetch_data(profile.id)?; // Auto-context
Ok(())
```

**Control attributes**: `#[no_context]`, `#[context(display)]`, `#[context(ignore)]`

### 2. Attribute Pattern Matching

**Feature flag**: `attributes` (included in `full`)

Extract values from attributes using `__unknown__` placeholder:

```rust,ignore
let input: syn::ItemStruct = parse_quote! {
    #[derive(Debug)]
    #[api_version(v2)]
    struct ApiRoutes {
        #[route(GET, "/users")]
        #[deprecated]
        list_users: String,

        #[route(POST, "/users")]
        create_user: String,

        #[route(GET, "/users/{id}")]
        get_user: String,

        unrouted_field: String,
    }
};

// 1. has_attributes! - Check if struct has required attributes
let is_valid_api = has_attributes!(input, #[derive(Debug)] #[api_version(v2)]);
assert!(is_valid_api);

// 2. get_attributes! - Extract API version from struct
let versions: Vec<TokenStream> = get_attributes!(input, #[api_version(__unknown__)]);
assert_eq!(versions[0].to_string(), "v2");

// 3. fields_with_attributes! - Filter only deprecated route fields
let deprecated: Vec<(usize, &Field)> =
    fields_with_attributes!(&input, #[route(GET, "/users")] #[deprecated]).collect();
assert_eq!(deprecated.len(), 1);

// 4. fields_get_attributes! - Extract HTTP methods from all routed fields
let routes: Vec<(usize, &Field, Vec<TokenStream>)> =
    fields_get_attributes!(&input, #[route(__unknown__, "/users")]);
assert_eq!(routes.len(), 2); // list_users and create_user
assert_eq!(routes[0].2[0].to_string(), "GET");
assert_eq!(routes[1].2[0].to_string(), "POST");

Ok(())
```

### 3. Exhaustive AST Traversal

**Feature flag**: `all-syntax-cases` (included in `full`)

Generate recursive handlers for all syn types:

```rust,ignore
all_syntax_cases! {
    setup => {
        generated_fn_prefix: "process",
        additional_input_type: &mut Context,
    }
    default_cases => {
        // Called for matching types across entire AST
        fn handle_expr(_expr: &mut syn::Expr, _ctx: &mut Context);

        #[after_system]  // Run after children processed
        fn finalize(_item: &mut syn::Item, _ctx: &mut Context);

        // Handle multiple syn types together (e.g., attributes + generics)
        fn check_attrs_and_generics(
            _attrs: &mut Vec<syn::Attribute>,
            _generics: &mut syn::Generics,
            _ctx: &mut Context
        );
    }
    special_cases => {
        // Override for specific variants
        fn handle_call(_call: &mut syn::ExprCall, _ctx: &mut Context);
    }
}

// Function implementations for the handlers
fn handle_expr(_expr: &mut syn::Expr, _ctx: &mut Context) {
    // Process expressions
}

fn finalize(_item: &mut syn::Item, _ctx: &mut Context) {
    // Finalize items after processing children
}

fn check_attrs_and_generics(
    _attrs: &mut Vec<syn::Attribute>,
    _generics: &mut syn::Generics,
    _ctx: &mut Context,
) {
    // Check attributes and generics together
}

fn handle_call(_call: &mut syn::ExprCall, _ctx: &mut Context) {
    // Handle function calls specially
}
```

Smart unwrapping of `Box<T>`, `Vec<T>`, `Punctuated<T, _>`. Generates handlers for Item, Expr, Stmt, Pat, Type, and more.

### 4. Helper Utilities

**Feature flags**: Individual helpers or `full` for all

```rust,ignore
use std::fs;

// Manual error context with file/line info
// Feature: `context` (included in `general` and `full`)
fn load_config() -> anyhow::Result<String> {
    fs::read_to_string("file.txt").with_context(context!("Loading config"))
}

// Token stream builder
// Feature: `tokens-builder` (included in `full`)
let mut tokens = TokensBuilder::default();
tokens.add(quote! { println!("Hello"); });
tokens.braced(); // Wrap in { }
let stream = tokens.finalize();
assert!(!stream.is_empty());

// Generate indexed names: field0, field1, field2
// Feature: `indexed-name` (included in `full`)
let names = indexed_name(syn::parse_quote!(field), 3);
assert_eq!(names.len(), 3);

// Find crates (handles renames)
// Feature: `find-crate` (included in `full`)
if let Some(path) = find_crate("quote", quote!()) {
    assert!(!path.to_string().is_empty());
}
// Returns first found crate or None
let async_rt = find_crate_list(&[("tokio", quote!()), ("async-std", quote!())]);
```

```rust,ignore
// Parse that returns Ok(...) with compile_error! on failure
// Feature: `parse-macro-input` (included in `full`)
let parsed = parse_macro_input!(input as syn::DeriveInput);
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

### 6. Add Code - Make (Docify) Examples minimal

**Feature flag**: `add-code` (included in `full`)

Use `#[add_code]` to inject setup/teardown or assertions around a function while keeping
[docify](https://crates.io/crates/docify)-generated examples clean and minimal.

```rust
use easy_macros::add_code;

#[add_code(before = { let _guard = setup(); })]
fn example() {
  // This stays minimal for docify examples.
  do_work();
}

// The code inside the braces is inserted without the braces.
#[add_code(after = {
  assert!(is_valid());
  Ok(())
})]
fn with_return() -> Result<(), Error> {
  do_more_work();
}

#[add_code(before = { let _guard = setup(); }, after = { teardown(); })]
fn both() {
  do_another_work();
}
```

## Feature Flags

### Feature Groups

- **`general`** - Automatic error context for any project
  - Includes: `always-context`, `context`
  - Use when you only need automatic error context, not proc-macro development tools

- **`full`** - Complete toolkit for proc-macro development
  - Includes: `all-syntax-cases`, `always-context`, `attributes`, `anyhow-result`, `add-code`, and all helpers
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
- **`add-code`** - `#[add_code]` for injecting code before/after function bodies

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
