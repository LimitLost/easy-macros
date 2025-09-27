//! ### Token Stream Management
//!
//! - [`TokensBuilder`] - Accumulate and combine token streams with methods inside
//! - [`readable_token_stream`] - Format token strings for better readability
//! - [`token_stream_to_consistent_string`] - Normalize token representation across contexts
//!
//! ### Error Handling
//!
//! - [`parse_macro_input!`] - Enhanced version of syn's macro that returns `Ok(TokenStream)` on parse errors (instead of `TokenStream`)
//! - [`expr_error_wrap`] with [`ErrorData`] trait - Wrap expressions with compile-time error reporting
//!
//! ### Code Generation Utilities
//!
//! - [`indexed_name`] - Generate indexed identifiers (`field0`, `field1`, etc.)
//! - [`find_crate`] - Locate crate references for generated code (supports renaming)
//! - [`find_crate_list`] - Try multiple crates, return first found
//!

mod indexed_name;
pub use indexed_name::indexed_name;

mod macro_result;
pub use macro_result::TokensBuilder;

mod expr_error_wrap;
pub use expr_error_wrap::{ErrorData, expr_error_wrap};

mod readable_token_stream;
pub use readable_token_stream::readable_token_stream;

mod find_crate;
pub use find_crate::{find_crate, find_crate_list};

mod token_stream_to_consistent_string;
pub use token_stream_to_consistent_string::*;

#[cfg(test)]
mod examples;

// This attribute is needed because this macro crashes cargo check in VS Code for some reason
#[cfg(test)]
docify::compile_markdown!("README.docify.md", "README.md");

#[macro_export]
/// Adjusted version of syn's `parse_macro_input!` macro for `#[macro_result]` procedural macro attribute.
///
/// Unlike syn's original `parse_macro_input!` macro, this version returns `Ok(TokenStream)`
/// on parse errors instead of returning a `TokenStream` directly. This makes
/// it more suitable for use in procedural macros which are returning `anyhow::Result<TokenStream>`. See `macro_result` macro.
///
/// # Behavior
///
/// On successful parsing, the macro returns the parsed value directly.
/// On parse errors, it returns `Ok(TokenStream)` containing the error as `compile_error!` tokens,
/// allowing the error to be displayed at compile time while still providing a valid return value.
///
/// # Syntax
///
/// ```ignore
/// // Parse as a specific type
/// let input = parse_macro_input!(tokens as DeriveInput);
///
/// // Parse with a custom parser
/// let input = parse_macro_input!(tokens with syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated);
///
/// // Parse with type inference
/// let input = parse_macro_input!(tokens);
/// ```
///
/// # Examples
///
/// ```ignore
/// use easy_macros::{parse_macro_input, macro_result};
/// use proc_macro::TokenStream;
/// use syn::DeriveInput;
///
/// #[proc_macro_derive(MyDerive)]
/// #[macro_result]
/// pub fn my_derive(input: TokenStream) -> anyhow::Result<TokenStream> {
///     // This will return compile errors automatically on parse failure
///     let input = parse_macro_input!(input as DeriveInput);
///     
///     // Your macro logic here...
///     Ok(quote::quote! {
///         // Generated code
///     }.into())
/// }
/// ```
///
/// # Advantages over syn's version
///
/// - Returns `Ok(TokenStream)` on errors instead of raw `TokenStream`
/// - Better integration with macros that return for example `anyhow::Result<TokenStream>`
///
/// # Parameters
///
/// - `$tokenstream` - The input `TokenStream` to parse
/// - `$ty` - The target type to parse into (with `as` syntax)
/// - `$parser` - A custom parser function (with `with` syntax)
macro_rules! parse_macro_input {
    ($tokenstream:ident as $ty:ty) => {
        match syn::parse::<$ty>($tokenstream) {
            syn::__private::Ok(data) => data,
            syn::__private::Err(err) => {
                return Ok(syn::__private::TokenStream::from(err.to_compile_error()));
            }
        }
    };
    ($tokenstream:ident with $parser:path) => {
        match syn::parse::Parser::parse($parser, $tokenstream) {
            syn::__private::Ok(data) => data,
            syn::__private::Err(err) => {
                return Ok(syn::__private::TokenStream::from(err.to_compile_error()));
            }
        }
    };
    ($tokenstream:ident) => {
        $crate::parse_macro_input!($tokenstream as _)
    };
}
