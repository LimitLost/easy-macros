//!
//! ## Key Features
//!
//! - **Attribute presence checking** with [`has_attributes!`]
//! - **Pattern extraction** with [`get_attributes!`] using `__unknown__` placeholders
//! - **Field-level filtering** with [`fields_with_attributes!`]
//! - **Field attribute extraction** with [`fields_get_attributes!`]
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use easy_macros_attributes::{has_attributes, get_attributes, fields_with_attributes};
//! use syn::parse_quote;
//!
//! let input = parse_quote! {
//!     #[derive(Debug)]
//!     #[api_version(v1)]
//!     struct User {
//!         #[validate]
//!         name: String,
//!     }
//! };
//!
//! // Check if attributes exist
//! let has_debug = has_attributes!(input, #[derive(Debug)]);
//!
//! // Extract dynamic parts using __unknown__
//! let versions: Vec<_> = get_attributes!(input, #[api_version(__unknown__)]);
//!
//! // Filter fields by attributes
//! let validated: Vec<_> = fields_with_attributes!(input, #[validate]).collect();
//! ```
//!
//! ## The `__unknown__` Pattern
//!
//! The `__unknown__` placeholder is a powerful feature that allows you to extract
//! dynamic content from attributes:
//!
//! ```rust,ignore
//! // Extract HTTP methods: GET, POST, DELETE, etc.
//! get_attributes!(item, #[route(__unknown__, "/users")])
//!
//! // Extract configuration values
//! get_attributes!(item, #[config(database_url = __unknown__)])
//!
//! // Partial identifier matching
//! get_attributes!(item, #[test_case___unknown__])  // matches test_case_1, test_case_foo, etc.
//! ```
//!
//! See individual macro documentation for detailed examples and usage patterns.

// Compile README.docify.md to README.md (in the attributes crate)
#[cfg(feature = "generate-readme")]
docify::compile_markdown!("README.docify.md", "../attributes/README.md");

#[cfg(test)]
mod examples;

mod data;
mod fields_get_attributes;
mod fields_with_attributes;
mod get_attributes;
mod has_attributes;

use always_context::always_context;
use anyhow_result::anyhow_result;
use helpers::find_crate_list;
use proc_macro::TokenStream;
use quote::quote;

fn root_macros_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[
        ("easy-lib", quote! {::macros}),
        ("easy-macros", quote! {::macros}),
        ("attributes", quote! {}),
        ("attributes-macros", quote! {}),
    ]) {
        found
    } else {
        quote! {}
    }
}

fn context_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[
        ("easy-lib", quote! {::helpers}),
        ("easy-macros", quote! {::helpers}),
    ]) {
        found
    } else {
        quote! {self}
    }
}

#[always_context]
#[proc_macro]
#[anyhow_result]
/// Checks if an item has all specified attributes.
///
/// Returns `true` if the passed in item has all specified attributes (one or more).
///
/// # Syntax
/// ```rust,ignore
/// has_attributes!(item, #[attribute1] #[attribute2] ... #[attributeN])
/// ```
///
/// # Arguments
/// * `item` - Any syntax node that has an `.attrs` field (e.g., struct, enum, function, field)
/// * `attributes` - One or more attributes to check for (all must be present)
///
/// # Return Value
/// Returns a boolean expression that evaluates to `true` if ALL specified attributes
/// are found on the item, `false` otherwise.
///
/// # Matching Behavior
/// **IMPORTANT**: This macro performs **exact matching** on attributes. The entire attribute
/// must match exactly, including all tokens and their structure.
///
/// - `#[derive(Debug)]` matches ONLY `#[derive(Debug)]`, NOT `#[derive(Debug, Clone)]`
/// - `#[serde(rename = "x")]` matches ONLY that exact attribute with that exact value
/// - Token structure must match exactly (whitespace is normalized and doesn't need to match)
///
/// # Examples
///
/// ## Basic Usage
#[doc = docify::embed!("src/examples.rs", has_attributes_basic_usage)]
///
/// ## Field Attributes
#[doc = docify::embed!("src/examples.rs", has_attributes_field_attributes)]
///
/// ## Exact Matching Gotchas
#[doc = docify::embed!("src/examples.rs", has_attributes_exact_matching)]
///
/// # Error Handling
/// This macro performs attribute parsing at compile time and will produce compile errors if:
/// - The `item` parameter doesn't have an `.attrs` field (e.g., not a valid syntax node)
/// - The attribute syntax is malformed (invalid Rust attribute syntax)
/// - No attributes are provided to check for
///
///
/// # Use Cases
/// - Validation in derive macros (checking for required companion attributes)
/// - Configuration checks in procedural macros
/// - Conditional code generation based on attribute presence
/// - Guard clauses to ensure required attributes exist
pub fn has_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    has_attributes::has_attributes(item)
}

// fn find_unknown(attr_template:&syn::Attribute,attr:syn::)

//Allow for only one unknown inside of attribute
// __unknown__ - unknown mark
//Example: #[attribute__unknown__]
//Example: #[attri__unknown__bute]
//Example: #[__unknown__attribute]
//Example: #[attribute(__unknown__)]
//Example: #[attribute(name=__unknown__)]
//Example: #[attribute = __unknown__]

#[always_context]
#[proc_macro]
#[anyhow_result]
/// Extracts dynamic values from attributes using `__unknown__` placeholders.
///
/// This macro allows pattern matching against attributes where some parts are unknown
/// and need to be extracted. Use `__unknown__` as a placeholder for the parts you want
/// to capture.
///
/// # Syntax
/// ```rust,ignore
/// get_attributes!(item, #[pattern_with___unknown__])
/// ```
///
/// # Arguments
/// * `item` - Any syntax node that has an `.attrs` field
/// * `pattern` - Attribute pattern with exactly one `__unknown__` placeholder
///
/// # Return Value
/// Returns `Vec<proc_macro2::TokenStream>` containing all the unknown replacements
/// found on the item. Each `TokenStream` represents the content that replaced `__unknown__`
/// in a matching attribute.
///
/// - **Empty vector `vec![]`**: No matching attributes found, or conditional attributes missing
/// - **Non-empty vector**: Each element is an extracted unknown replacement
/// - **Ordering**: Matches appear in the same order as attributes on the item
///
/// # `__unknown__` Placement Rules
/// 1. **Exactly one per pattern**: Only one `__unknown__` is allowed per attribute pattern
/// 2. **Flexible positioning**: Can appear anywhere in the attribute
/// 3. **Partial matching**: Can match parts of identifiers or literals
/// 4. **Requires exact match**: All non-unknown parts must match exactly
///
/// # Examples
///
/// ## Basic Value Extraction
#[doc = docify::embed!("src/examples.rs", get_attributes_basic_value_extraction)]
///
/// ## Partial Identifier Matching
#[doc = docify::embed!("src/examples.rs", get_attributes_partial_identifier_matching)]
///
/// ## Function Parameter Extraction
#[doc = docify::embed!("src/examples.rs", get_attributes_function_parameter_extraction)]
///
/// ## Complex Nested Matching
#[doc = docify::embed!("src/examples.rs", get_attributes_nested_example)]
///
/// ## Conditional Extraction with Multiple Attributes
#[doc = docify::embed!("src/examples.rs", get_attributes_conditional_extraction)]
///
/// # Error Handling
/// - **Compile Error**: if no `__unknown__` placeholder is found in any attribute
/// - **Compile Error**: if multiple `__unknown__` placeholders are used in a single pattern  
/// - **Returns `vec![]`**: if no matching attributes are found on the item
/// - **Returns `vec![]`**: if conditional attributes (non-unknown) are missing
/// - Provides detailed error context via `anyhow::Error` for debugging
///
/// # Error Examples
///
/// ## Multiple Unknowns (Compile Error)
/// ```rust,compile_fail
/// # use attributes::get_attributes;
/// # use syn::parse_quote;
/// # let input: syn::ItemStruct = parse_quote! { struct Foo; };
/// // ❌ This will fail to compile!
/// let invalid = get_attributes!(
///     input,
///     #[route(__unknown__, __unknown__)]
/// );
/// // Error: Multiple unknowns found in attributes!
/// ```
///
/// ## No Unknown (Compile Error)  
/// ```rust,compile_fail
/// # use attributes::get_attributes;
/// # use syn::parse_quote;
/// # let input: syn::ItemStruct = parse_quote! { struct Foo; };
/// // ❌ This will fail to compile!
/// let invalid = get_attributes!(
///     input,
///     #[derive(Debug)]
/// );
/// // Error: No unknown found in (to search for) attributes!
/// ```
///
/// ## No Matches (Returns Empty Vec)
#[doc = docify::embed!("src/examples.rs", error_handling_no_matches_example)]
///
/// ## Conditional Missing (Returns Empty Vec)  
#[doc = docify::embed!("src/examples.rs", error_handling_conditional_missing_example)]
///
/// # Common Mistakes
///
///
/// ## Exact Matching Required
#[doc = docify::embed!("src/examples.rs", get_attributes_exact_matching_required)]
///
/// # Use Cases
/// - Configuration extraction from attributes
/// - Building route handlers from attribute metadata
/// - Extracting validation rules
/// - Code generation based on attribute parameters
/// - Creating domain-specific languages in attributes
pub fn get_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    get_attributes::get_attributes(item)
}

#[always_context]
#[proc_macro]
#[anyhow_result]
/// Filters struct/enum fields by their attributes.
///
/// This macro examines the fields of a struct and returns an iterator over
/// fields that contain ALL of the specified attributes. Supports borrowing patterns
/// to control ownership of the returned fields.
///
/// **Note**: This macro uses [`has_attributes!`] internally, which performs **exact**
/// attribute matching. See [`has_attributes!`] documentation for matching behavior details.
///
/// # Syntax
/// ```rust,ignore
/// fields_with_attributes!(item, #[attr1] #[attr2] ... #[attrN])
/// fields_with_attributes!(&item, #[attr1] #[attr2])      // immutable borrow
/// fields_with_attributes!(&mut item, #[attr1] #[attr2])  // mutable borrow
/// ```
///
/// # Arguments
/// * `item` - A struct (optionally borrowed) that has a `.fields` field
/// * `attributes` - One or more attributes that must ALL be present on a field (exact match)
///
/// # Return Value
/// Returns an iterator over `(usize, Field)` tuples where:
/// - `usize` is the 0-based index of the field (0 for first field, 1 for second, etc.)
/// - `Field` is `syn::Field`, `&syn::Field`, or `&mut syn::Field` depending on borrowing
///
/// # Borrowing Behavior
/// - **No prefix**: `fields.into_iter()` - consumes the fields, returns owned `syn::Field`
/// - **`&` prefix**: `fields.iter()` - immutable references, returns `&syn::Field`
/// - **`&mut` prefix**: `fields.iter_mut()` - mutable references, returns `&mut syn::Field`
///
/// # Examples
///
/// ## Basic Field Filtering
#[doc = docify::embed!("src/examples.rs", fields_with_attributes_basic_filtering)]
///
/// ## Multiple Attribute Requirements (Exact Match)
#[doc = docify::embed!("src/examples.rs", fields_with_attributes_multiple_requirements)]
///
/// **Note**: Exact matching means `#[serde(rename = "user_name", skip_serializing_if = "Option::is_none")]`
/// won't match `#[serde(rename = "user_name")]` because it has additional content.
///
/// ## Borrowing to Preserve Original
#[doc = docify::embed!("src/examples.rs", fields_with_attributes_borrowing)]
///
/// # Error Handling
/// This macro will produce compile errors if:
/// - The `item` parameter doesn't have a `.fields` field
/// - The attribute syntax is malformed
/// - No attributes are provided to match against
///
/// The macro returns an iterator, so no fields matching the criteria simply results in an empty iterator (not an error).
///
pub fn fields_with_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    fields_with_attributes::fields_with_attributes(item)
}

#[always_context]
#[no_context]
#[proc_macro]
#[anyhow_result]
/// Debug version of `fields_with_attributes!` that panics with the result.
#[doc(hidden)]
pub fn fields_with_attributes_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let result = fields_with_attributes::fields_with_attributes(item)?;
    panic!("{result}",);
}

#[always_context]
#[proc_macro]
#[anyhow_result]
/// Extracts dynamic values from field attributes using `__unknown__` placeholders.
///
/// This macro combines field filtering with attribute pattern extraction. It examines
/// struct/enum fields and returns those that match the attribute pattern, along with
/// the extracted unknown parts from their attributes.
///
/// **Note**: This uses [`get_attributes!`] internally for pattern matching. A field can
/// have multiple matching attributes, resulting in multiple extracted values for that field.
///
/// # Syntax
/// ```rust,ignore
/// fields_get_attributes!(item, #[pattern_with___unknown__])
/// fields_get_attributes!(&item, #[pattern_with___unknown__])      // immutable borrow
/// fields_get_attributes!(&mut item, #[pattern_with___unknown__])  // mutable borrow
/// ```
///
/// # Arguments
/// * `item` - A struct (optionally borrowed) that has a `.fields` field
/// * `pattern` - Attribute pattern with exactly one `__unknown__` placeholder
///
/// # Return Value
/// Returns `Vec<(usize, Field, Vec<proc_macro2::TokenStream>)>` where:
/// - `usize` is the 0-based index of the field
/// - `Field` is `syn::Field`, `&syn::Field`, or `&mut syn::Field` depending on borrowing
/// - `Vec<proc_macro2::TokenStream>` contains all unknown replacements found on that field
///
/// **Important**: A single field can have multiple matching attributes, so the `Vec<TokenStream>`
/// can contain multiple elements. If a field has no matching attributes, it won't appear in the results.
///
/// # Error Handling
/// - If any field causes an error during extraction, the macro will return that error
/// - See [`get_attributes!`] for specific error conditions
/// - Fields that don't match the pattern are silently skipped (not an error)
///
/// # Examples
///
/// ## Route Configuration Extraction
#[doc = docify::embed!("src/examples.rs", fields_get_attributes_route_extraction)]
///
/// ## Database Column Configuration
#[doc = docify::embed!("src/examples.rs", fields_get_attributes_database_columns)]
///
/// ## Validation Rule Extraction
#[doc = docify::embed!("src/examples.rs", fields_get_attributes_validation_rules)]
///
/// ## Multiple Matching Attributes Per Field (Important!)
#[doc = docify::embed!("src/examples.rs", fields_get_attributes_multiple_matches_per_field)]
///
/// ## Borrowing for Memory Efficiency
#[doc = docify::embed!("src/examples.rs", fields_get_attributes_borrowing)]
///
/// ## Complex Pattern Matching
#[doc = docify::embed!("src/examples.rs", fields_get_attributes_complex_pattern)]
///
/// # Error Handling
/// - **Compile Error**: if no `__unknown__` placeholder is found in the pattern
/// - **Compile Error**: if multiple `__unknown__` placeholders are used  
/// - **Compile Error**: if the `item` parameter doesn't have a `.fields` field
/// - **Returns `vec![]`**: if no fields match the attribute pattern
/// - **Runtime Error**: if attribute parsing fails for any field (propagated via `anyhow::Error`)
///
/// If any field causes a parsing error, the entire macro invocation fails with a detailed error message
/// including the problematic field's information.
///
/// # Use Cases
/// - **Derive macro implementations**: Generate code based on field attributes (e.g., route handlers, database columns)
/// - **Configuration extraction**: Pull metadata from field attributes for code generation
/// - **Validation rule collection**: Gather all validation attributes across struct fields
/// - **API endpoint generation**: Build routing tables from field-level route attributes
/// - **ORM mapping**: Extract database column configurations from field attributes
/// - **Serialization customization**: Process field-level serialization directives
///
pub fn fields_get_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    fields_get_attributes::fields_get_attributes(item)
}

#[always_context]
#[no_context]
#[proc_macro]
#[anyhow_result]
/// Debug version of `fields_get_attributes!` that panics with the result.
#[doc(hidden)]
pub fn fields_get_attributes_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let result = fields_get_attributes::fields_get_attributes(item)?;
    panic!("{result}",);
}
