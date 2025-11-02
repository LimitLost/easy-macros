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

```rust,ignore
use syn::parse_quote;

// Check for a single attribute
let input: syn::ItemStruct = parse_quote! {
    #[derive(Debug)]
    #[serde(rename_all = "camelCase")]
    struct User {
        name: String,
    }
};

let has_debug = has_attributes!(input, #[derive(Debug)]);
assert!(has_debug);

// Check for multiple attributes (all must be present)
let has_both = has_attributes!(
    input,
    #[derive(Debug)] #[serde(rename_all = "camelCase")]
);
assert!(has_both);

// This returns false since #[derive(Clone)] is not present
let has_clone = has_attributes!(input, #[derive(Clone)]);
assert!(!has_clone);
```

### Pattern Matching with `get_attributes!` and `__unknown__`

The `__unknown__` placeholder allows you to extract dynamic parts from attributes:

#### Basic Value Extraction

```rust,ignore
use syn::parse_quote;

let input: syn::ItemStruct = parse_quote! {
    #[serde(rename = "custom_name")]
    #[serde(rename = "other_name")]
    struct User {
        name: String,
    }
};

// Extract rename values
let renames: Vec<proc_macro2::TokenStream> = get_attributes!(
    input,
    #[serde(rename = __unknown__)]
);

assert_eq!(renames.len(), 2);
assert_eq!(renames[0].to_string(), "\"custom_name\"");
assert_eq!(renames[1].to_string(), "\"other_name\"");
Ok(())
```

#### Partial Identifier Matching

```rust,ignore
use syn::parse_quote;

let input: syn::ItemStruct = parse_quote! {
    #[test_case_one]
    #[test_case_two]
    #[test_case_foo]
    #[other_attr]
    struct TestSuite;
};

// Extract the suffix after "test_case_"
let test_cases: Vec<proc_macro2::TokenStream> = get_attributes!(
    input,
    #[test_case___unknown__]
);

assert_eq!(test_cases.len(), 3);
assert_eq!(test_cases[0].to_string(), "one");
assert_eq!(test_cases[1].to_string(), "two");
assert_eq!(test_cases[2].to_string(), "foo");
Ok(())
```

### Field-Level Operations with `fields_with_attributes!`

Filter struct fields based on their attributes:

#### Basic Field Filtering

```rust,ignore
use attributes::fields_with_attributes;
use syn::parse_quote;

let input: syn::ItemStruct = parse_quote! {
    struct User {
        #[serde(skip)]
        id: u64,

        #[validate]
        #[serde(rename = "user_name")]
        name: String,

        #[validate]
        email: String,

        created_at: String,
    }
};

// Get fields with validation attributes
let validated_fields: Vec<(usize, syn::Field)> = fields_with_attributes!(
    input,
    #[validate]
)
.collect();

assert_eq!(validated_fields.len(), 2); // name and email fields
assert_eq!(validated_fields[0].0, 1); // name is at index 1
assert_eq!(validated_fields[1].0, 2);
```

#### Multiple Attribute Requirements

```rust,ignore
use attributes::fields_with_attributes;
use syn::parse_quote;

let input: syn::ItemStruct = parse_quote! {
    struct User {
        #[serde(skip)]
        id: u64,

        #[validate]
        #[serde(rename = "user_name")]
        name: String,

        #[validate]
        email: String,
    }
};

// Get fields that have BOTH validate AND the exact serde attribute
let validated_serde_fields: Vec<(usize, syn::Field)> = fields_with_attributes!(
    input,
    #[validate] #[serde(rename = "user_name")]
)
.collect();

assert_eq!(validated_serde_fields.len(), 1);
```

**Note**: Exact matching means `#[serde(rename = "user_name", skip_serializing_if = "Option::is_none")]`
won't match `#[serde(rename = "user_name")]` because it has additional content.

### Advanced Pattern Extraction with `fields_get_attributes!`

Extract dynamic values from field attributes:

#### Route Configuration Extraction

```rust,ignore
use attributes::{fields_get_attributes, fields_get_attributes_debug};
use syn::parse_quote;

let input: syn::ItemStruct = parse_quote! {
    struct ApiEndpoints {
        #[route(GET, "/users")]
        get_users: String,

        #[route(POST, "/users")]
        create_user: String,

        #[route(GET, "/users/{id}")]
        get_user: String,

        #[route(DELETE, "/users/{id}")]
        delete_user: String,

        #[other_attr]
        non_route_field: String,
    }
};

// Extract HTTP methods for all route fields
let methods: Vec<(usize, syn::Field, Vec<proc_macro2::TokenStream>)> =
    fields_get_attributes!(input, #[route(__unknown__, "/users")]);

assert_eq!(methods.len(), 2); // get_users and create_user
assert_eq!(methods[0].2[0].to_string(), "GET"); // get_users method
assert_eq!(methods[1].2[0].to_string(), "POST"); // create_user method

Ok(())
```

#### Database Column Configuration

```rust,ignore
use attributes::fields_get_attributes;
use syn::parse_quote;

let input: syn::ItemStruct = parse_quote! {
    struct UserTable {
        #[column(id, primary_key)]
        #[column(id, auto_increment)]
        id: i32,

        #[column(varchar, length = 255)]
        #[unique]
        email: String,

        #[column(varchar, length = 100)]
        #[nullable]
        name: Option<String>,

        #[column(timestamp)]
        created_at: String,
    }
};

// Extract column types
let column_types: Vec<(usize, syn::Field, Vec<proc_macro2::TokenStream>)> =
    fields_get_attributes!(input, #[column(__unknown__, length = 255)]);

assert_eq!(column_types.len(), 1); // only email field
assert_eq!(column_types[0].2[0].to_string(), "varchar");

Ok(())
```

### Complex Pattern Matching Scenarios

#### Nested Attribute Matching

```rust,ignore
use syn::parse_quote;

// Docify has trouble parsing parse_quote! with attributes inside,
// so we use a string literal workaround
let input: syn::ItemStruct = syn::parse_str(
    r#"
    #[config(database(url = "postgres://localhost"))]
    #[config(redis(url = "redis://localhost"))]
    struct AppConfig;
"#,
)?;

// Extract database URL
let db_urls: Vec<proc_macro2::TokenStream> = get_attributes!(
    input,
    #[config(database(url = __unknown__))]
);
assert_eq!(db_urls[0].to_string(), "\"postgres:
```

#### Multiple Matching Attributes Per Field

```rust,ignore
use attributes::fields_get_attributes;
use syn::parse_quote;

// **KEY CONCEPT**: A single field can have MULTIPLE matching attributes!
// Each matching attribute on the same field adds to the Vec<TokenStream>
let input: syn::ItemStruct = parse_quote! {
    struct Multi {
        #[tag(v1)]  // ← All three of these match #[tag(__unknown__)]
        #[tag(v2)]  // ←
        #[tag(v3)]  // ←
        field: String,
    }
};

let versions: Vec<(usize, syn::Field, Vec<proc_macro2::TokenStream>)> =
    fields_get_attributes!(input, #[tag(__unknown__)]);

assert_eq!(versions.len(), 1); // ONE field in results
assert_eq!(versions[0].2.len(), 3); // THREE extracted values from that field
assert_eq!(versions[0].2[0].to_string(), "v1");
assert_eq!(versions[0].2[1].to_string(), "v2");
assert_eq!(versions[0].2[2].to_string(), "v3");

Ok(())
```

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

```rust,ignore
use syn::parse_quote;

let input: syn::ItemStruct = parse_quote! {
    #[derive(Debug)]
    struct User;
};

// No #[route(...)] attributes exist, so this returns empty
let no_routes: Vec<proc_macro2::TokenStream> = get_attributes!(
    input,
    #[route(__unknown__)]
);
assert_eq!(no_routes.len(), 0);
Ok(())
```

### Conditional Attributes Missing

```rust,ignore
use syn::parse_quote;

// Missing the required #[derive(Debug)] attribute
let input_no_debug: syn::ItemStruct = parse_quote! {
    #[api_version(v1)]
    struct User;
};

// Without derive(Debug), this returns empty vec![]
let no_versions: Vec<proc_macro2::TokenStream> = get_attributes!(
    input_no_debug,
    #[derive(Debug)] #[api_version(__unknown__)]
);
assert_eq!(no_versions.len(), 0); // Empty because derive(Debug) is missing
Ok(())
```

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

## When to Use This Crate

**Perfect for:**
- Building derive macros that need attribute-based configuration
- Creating code generators that parse custom attributes
- Implementing domain-specific languages embedded in attributes
- Extracting metadata from attributes for code analysis
- Building validation or serialization libraries

**Consider alternatives for:**
- Simple attribute presence checking (built-in `cfg` attributes might suffice)
- Runtime attribute access (use reflection crates instead)
- Non-procedural macro contexts

---

**Note**: All examples in this README are tested as part of the test suite. See `src/examples.rs` for the full, runnable code.
