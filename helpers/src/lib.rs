//! ### General Use (not only for macros)
//!
//! - [`context!`] - Generates context strings for error handling with automatic file/line information (requires `context` feature)
//!
//! ### Token Stream Management
//!
//! - [`TokensBuilder`] - Accumulate and combine token streams with methods inside
//! - [`readable_token_stream`] - Format token strings for better readability
//! - [`token_stream_to_consistent_string`] - Normalize token representation across contexts
//!
//! ### Error Handling
//!
//! - [`parse_macro_input!`] - Enhanced version of syn's macro that returns `Ok(TokenStream)` on parse errors (instead of `TokenStream`)
//! - [`expr_error_wrap`] with [`CompileErrorProvider`] trait - Wrap expressions with compile-time error reporting
//!
//! ### Code Generation Utilities
//!
//! - [`indexed_name`] - Generate indexed identifiers (`field0`, `field1`, etc.)
//! - [`find_crate`] - Locate crate references for generated code (supports renaming)
//! - [`find_crate_list`] - Try multiple crates, return first found
//!

#[cfg(feature = "context")]
mod context;
#[cfg(feature = "context")]
pub use context::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "full")]
mod indexed_name;
#[cfg(feature = "full")]
pub use indexed_name::indexed_name;

#[cfg(feature = "full")]
mod tokens_builder;
#[cfg(feature = "full")]
pub use tokens_builder::TokensBuilder;

#[cfg(feature = "full")]
mod expr_error_wrap;
#[cfg(feature = "full")]
pub use expr_error_wrap::{CompileErrorProvider, expr_error_wrap};

#[cfg(feature = "full")]
mod readable_token_stream;
#[cfg(feature = "full")]
pub use readable_token_stream::readable_token_stream;

#[cfg(feature = "full")]
mod find_crate;
#[cfg(feature = "full")]
pub use find_crate::{find_crate, find_crate_list};

#[cfg(feature = "full")]
mod token_stream_to_consistent_string;
#[cfg(feature = "full")]
pub use token_stream_to_consistent_string::*;

#[cfg(feature = "full")]
mod parse_macro_input;

#[cfg(test)]
mod examples;

// This attribute is needed because this macro crashes cargo check in VS Code for some reason
#[cfg(test)]
docify::compile_markdown!("README.docify.md", "README.md");
