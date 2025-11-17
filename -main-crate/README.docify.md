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

<!-- docify::embed!("src/examples.rs", always_context_example) -->

**Control attributes**: `#[no_context]`, `#[context(display)]`, `#[context(ignore)]`

### 2. Attribute Pattern Matching

**Feature flag**: `attributes` (included in `full`)

Extract values from attributes using `__unknown__` placeholder:

<!-- docify::embed!("src/examples.rs", attributes_comprehensive_example) -->

### 3. Exhaustive AST Traversal

**Feature flag**: `all-syntax-cases` (included in `full`)

Generate recursive handlers for all syn types:

<!-- docify::embed!("src/examples.rs", ast_traversal_example) -->

Smart unwrapping of `Box<T>`, `Vec<T>`, `Punctuated<T, _>`. Generates handlers for Item, Expr, Stmt, Pat, Type, and more.

### 4. Helper Utilities

**Feature flags**: Individual helpers or `full` for all

<!-- docify::embed!("src/examples.rs", helper_utilities_comprehensive_example) -->

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
