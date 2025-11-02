# What is this?

[![Crates.io](https://img.shields.io/crates/v/easy-macros-attributes.svg)](https://crates.io/crates/easy-macros-attributes)
[![Documentation](https://docs.rs/easy-macros-attributes/badge.svg)](https://docs.rs/easy-macros-attributes)

Easy Macros Attributes is a specialized crate within the [Easy Macros ecosystem](https://crates.io/crates/easy_macros) that provides powerful procedural macros for working with Rust attributes. It enables pattern matching, extraction, and filtering based on attributes in your code, with support for unknown placeholders and flexible matching patterns.

## Core Features

### Attribute Checking

- [`has_attributes!`](https://docs.rs/easy-macros-attributes/latest/easy_macros_attributes/macro.has_attributes.html) - Check if an item has all specified attributes

### Attribute Pattern Matching with Unknowns

- [`get_attributes!`](https://docs.rs/easy-macros-attributes/latest/easy_macros_attributes/macro.get_attributes.html) - Extract dynamic values from attributes using `__unknown__` placeholders

### Field-Level Attribute Operations

- [`fields_with_attributes!`](https://docs.rs/easy-macros-attributes/latest/easy_macros_attributes/macro.fields_with_attributes.html) - Filter struct/enum fields by their attributes
- [`fields_get_attributes!`](https://docs.rs/easy-macros-attributes/latest/easy_macros_attributes/macro.fields_get_attributes.html) - Extract dynamic values from field attributes

### Advanced Pattern Matching

- [`AttrWithUnknown`](https://docs.rs/easy-macros-attributes/latest/easy_macros_attributes/struct.AttrWithUnknown.html) - Advanced attribute parsing with unknown placeholder support

## Examples

### Basic Attribute Checking with `has_attributes!`

<!-- docify::embed!("src/examples.rs", has_attributes_basic_usage) -->

### Pattern Matching with `get_attributes!` and `__unknown__`

The `__unknown__` placeholder allows you to extract dynamic parts from attributes:

#### Basic Value Extraction

<!-- docify::embed!("src/examples.rs", get_attributes_basic_value_extraction) -->

#### Partial Identifier Matching

<!-- docify::embed!("src/examples.rs", get_attributes_partial_identifier_matching) -->

### Field-Level Operations with `fields_with_attributes!`

Filter struct fields based on their attributes:

#### Basic Field Filtering

<!-- docify::embed!("src/examples.rs", fields_with_attributes_basic_filtering) -->

#### Multiple Attribute Requirements

<!-- docify::embed!("src/examples.rs", fields_with_attributes_multiple_requirements) -->

**Note**: Exact matching means `#[serde(rename = "user_name", skip_serializing_if = "Option::is_none")]`
won't match `#[serde(rename = "user_name")]` because it has additional content.

### Advanced Pattern Extraction with `fields_get_attributes!`

Extract dynamic values from field attributes:

#### Route Configuration Extraction

<!-- docify::embed!("src/examples.rs", fields_get_attributes_route_extraction) -->

#### Database Column Configuration

<!-- docify::embed!("src/examples.rs", fields_get_attributes_database_columns) -->

### Complex Pattern Matching Scenarios

#### Nested Attribute Matching

<!-- docify::embed!("src/examples.rs", get_attributes_nested_example) -->

#### Multiple Matching Attributes Per Field

<!-- docify::embed!("src/examples.rs", fields_get_attributes_multiple_matches_per_field) -->

### Integration in Procedural Macros

```rust,ignore
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use easy_macros_attributes::{fields_with_attributes, fields_get_attributes};

#[proc_macro_derive(ApiEndpoints)]
pub fn derive_api_endpoints(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Get all fields with route attributes, extracting HTTP methods
    let routes: Vec<(usize, syn::Field, Vec<proc_macro2::TokenStream>)> =
        fields_get_attributes!(input, #[route(__unknown__)]);

    let route_implementations = routes.iter().map(|(_, field, methods)| {
        let field_name = &field.ident;
        // Note: A field might have multiple route attributes with different methods
        let method_impls = methods.iter().map(|method| {
            quote! {
                pub fn #field_name() -> Route {
                    Route::new(stringify!(#method), "/path")
                }
            }
        });

        quote! {
            #(#method_impls)*
        }
    });

    quote! {
        impl ApiEndpoints for YourStruct {
            #(#route_implementations)*
        }
    }.into()
}
```

## Understanding `__unknown__` Placeholders

The `__unknown__` placeholder is a powerful feature that allows pattern matching in attributes:

### Placement Rules

1. **Single unknown per attribute pattern**: Only one `__unknown__` is allowed per attribute pattern
2. **Flexible positioning**: Can appear in identifiers, literals, or as standalone tokens
3. **Partial matching**: Can match parts of identifiers (e.g., `prefix___unknown___suffix`)

### Matching Behavior

- **Exact match required**: All non-unknown parts must match exactly
- **Multiple captures**: Same pattern can match multiple attributes, collecting all unknowns
- **Type preservation**: Unknown content is captured as `proc_macro2::TokenStream`

### Common Patterns

```rust,ignore
// Method extraction
#[route(__unknown__, "/path")]     // Captures HTTP method

// Path parameter extraction
#[route(GET, __unknown__)]         // Captures full path

// Partial identifier matching
#[test___unknown__]                // Captures suffix of test attributes

// Value extraction from key-value pairs
#[config(key = __unknown__)]       // Captures configuration values

// Nested structure matching
#[outer(inner(__unknown__))]       // Captures inner content
```

## Error Handling

All macros return empty results or compile errors with detailed messages:

### No Matches (Returns Empty)

<!-- docify::embed!("src/examples.rs", error_handling_no_matches_example) -->

### Conditional Attributes Missing

<!-- docify::embed!("src/examples.rs", error_handling_conditional_missing_example) -->

For more error handling examples, see the [documentation](https://docs.rs/easy-macros-attributes/latest/easy_macros_attributes/).

## Performance Considerations

- **Compile-time execution**: All processing happens at compile time
- **Efficient pattern matching**: Uses optimized string matching for unknown detection
- **Minimal runtime overhead**: Generated code has zero runtime cost
- **Caching**: Repeated patterns are efficiently processed

## Integration with Easy Macros Ecosystem

This crate is part of the larger Easy Macros ecosystem and integrates seamlessly with other components:

- **[Easy Macros Helpers](https://crates.io/crates/easy-macros-helpers)**: Provides underlying utilities
- **[Easy Macros](https://crates.io/crates/easy_macros)**: Main entry point with all features
- **Context generation**: Automatic error context using [`context!`](https://docs.rs/easy-macros-helpers/latest/easy_macros_helpers_macro_safe/macro.context.html)
- **Token stream utilities**: Built on [`TokensBuilder`](https://docs.rs/easy-macros-helpers/latest/easy_macros_helpers_macro_safe/struct.TokensBuilder.html)

---

**Note**: All examples in this README are tested as part of the test suite. See `src/examples.rs` for the full, runnable code.
