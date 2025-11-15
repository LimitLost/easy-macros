# Easy Macros

[![Crates.io](https://img.shields.io/crates/v/easy-macros.svg)](https://crates.io/crates/easy-macros)
[![Documentation](https://docs.rs/easy-macros/badge.svg)](https://docs.rs/easy-macros)
[![License](https://img.shields.io/crates/l/easy-macros.svg)](https://github.com/LimitLost/easy-macros/blob/master/LICENSE)

Automatic error context for any Rust project + powerful procedural macro utilities.

## General feature flags

```toml
# Only automatic error context
[dependencies]
easy-macros = { version = "...", features = ["general"] }

# All tools
[dependencies]
easy-macros = { version = "...", features = ["for-macro"] }
```

## Features

### 1. Automatic Error Context - **Works in Any Project**

Add `.with_context()` to all `?` operators automatically:

```rust
use easy_macros::macros::always_context;
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

Extract values from attributes using `__unknown__` placeholder:

```rust
use easy_macros::macros::{get_attributes, fields_get_attributes};

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

Generate recursive handlers for all syn types:

```rust
use easy_macros::macros::all_syntax_cases;

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

```rust
use easy_macros::helpers::*;

// Manual error context with file/line info
fs::read("file.txt").with_context(context!("Loading config"))?;

// Token stream builder
let mut tokens = TokensBuilder::default();
tokens.add(quote! { println!("Hello"); });
tokens.braced();  // Wrap in { }

// Parse with compile_error! on failure
let parsed = parse_macro_input!(input as syn::DeriveInput);

// Generate indexed names: field0, field1, field2
let names = indexed_name(syn::parse_quote!(field), 3);

// Find crates (handles renames)
let path = find_crate("serde", quote!(::Serialize))?;
let async_rt = find_crate_list(&[("tokio", quote!()), ("async-std", quote!())])?;
```

### 5. Result Type for Proc Macros

Use `anyhow::Result<TokenStream>` in proc macros:

```rust
use easy_macros::macros::anyhow_result;

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
    macros::{anyhow_result, always_context, fields_get_attributes},
    helpers::{TokensBuilder, parse_macro_input},
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

- **`general`** - Error context (`#[always_context]`, `context!`) for any project
- **`for-macro`** - All tools for proc-macro development (includes `general`)
- **`build`** - Build macro that auto-adds `#[always_context]` to all functions returning `anyhow::Result`
- **`easy-sql`** - Add Easy-Sql crate integration for `#[always_context]`

## License

Apache License, Version 2.0 - See [LICENSE](LICENSE)
