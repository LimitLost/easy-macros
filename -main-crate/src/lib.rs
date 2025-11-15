#[cfg(feature = "all-syntax-cases")]
pub use all_syntax_cases::*;

#[cfg(feature = "always-context")]
pub use always_context::*;

#[cfg(feature = "build")]
pub use always_context_build;

#[cfg(feature = "anyhow-result")]
/// Enables procedural macros to return `anyhow::Result<TokenStream>` for ergonomic error handling.
///
/// This attribute wraps proc-macro functions to automatically handle `anyhow::Result` return types,
/// converting errors into appropriate `compile_error!` tokens.
///
/// # Usage
///
/// ```rust,ignore
/// use anyhow::Context;
/// use proc_macro::TokenStream;
///
/// #[proc_macro]
/// #[anyhow_result]
/// pub fn my_macro(input: TokenStream) -> anyhow::Result<TokenStream> {
///     let parsed: syn::ItemStruct = syn::parse(input)
///         .context("Expected a struct definition")?;
///     
///     // Your macro logic here
///     Ok(quote! { /* generated code */ }.into())
/// }
/// ```
///
/// # Error Handling
///
/// When your function returns an `Err`, `anyhow_result` automatically converts it:
/// - **`#[proc_macro]` and `#[proc_macro_derive]`**: Returns `compile_error!` with the error message
/// - **`#[proc_macro_attribute]`**: Returns `compile_error!` followed by the original input item
///
/// # See Also
///
/// - [`anyhow`](https://docs.rs/anyhow/) - Error handling library
/// - [`syn`](https://docs.rs/syn/) - Rust code parsing  
/// - [`quote`](https://docs.rs/quote/) - Code generation
pub use anyhow_result::anyhow_result;

#[cfg(feature = "attributes")]
pub use attributes::*;

// === Helper Function Exports ===

/// Helper utilities for building procedural macros
#[cfg(feature = "helpers-dont-use-directly-this-feature")]
pub use helpers::*;

#[cfg(test)]
mod macro_tests;
