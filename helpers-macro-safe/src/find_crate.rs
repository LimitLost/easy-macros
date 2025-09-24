use proc_macro_crate::FoundCrate;
use proc_macro2::{Span, TokenStream};
use quote::quote;

/// Locates a crate in the current Cargo.toml and generates the appropriate path reference.
///
/// Uses [`proc_macro_crate::crate_name`](https://docs.rs/proc-macro-crate/latest/proc_macro_crate/fn.crate_name.html) under the hood to determine how to reference the crate.
///
/// # Crate Renaming Support
///
/// The function properly handles crates that have been renamed in Cargo.toml using the `package` directive:
///
/// ```toml
/// [dependencies]
/// my_renamed_crate = { package = "original-crate-name", version = "1.0" }
/// some_crate = "2.0"  # Normal dependency
/// ```
///
/// When searching for `"original-crate-name"`, this function will return `my_renamed_crate::...`
/// because that's the actual import name that should be used in the generated code.
///
/// # Arguments
///
/// * `crate_name` - The original name of the crate (the `package` name, not the renamed dependency name)
/// * `after_name` - Additional path segments to append after the crate name
///
/// # Returns
///
/// * `Some(TokenStream)` - The path to the crate with the suffix appended
/// * `None` - If the crate is not found in the current Cargo.toml
///
/// # Examples
///
/// ```rust
/// use easy_macros_helpers_macro_safe::find_crate;
/// use quote::quote;
///
/// // Find a crate without additional path
/// if let Some(path) = find_crate("serde", quote!()) {
///     // Returns: `serde` or `crate` depending on context
/// }
///
/// // Find a crate with additional path segments
/// if let Some(path) = find_crate("my_crate", quote!(::utils::helper)) {
///     // Returns: `my_crate::utils::helper` or `crate::utils::helper`
/// }
///
/// // With a renamed crate in Cargo.toml:
/// // [dependencies]
/// // serde_renamed = { package = "serde", version = "1.0" }
/// if let Some(path) = find_crate("serde", quote!(::Serialize)) {
///     // Returns: `serde_renamed::Serialize`
/// }
/// ```
///
/// # Use Cases
///
/// - Building qualified paths for generated code
/// - Handling re-exports and crate renaming
/// - Supporting users who rename dependencies to avoid conflicts
///
/// # Generated Output Examples
///
/// When `after_name` is empty:
/// - `crate` (if the macro is used within its own crate)
/// - `your_crate_name` (if used as an external dependency)
/// - `renamed_name` (if the crate was renamed in Cargo.toml)
///
/// When `after_name` is `::utils::helper`:
/// - `crate::utils::helper` (within own crate)
/// - `your_crate_name::utils::helper` (as external dependency)
/// - `renamed_name::utils::helper` (if renamed in Cargo.toml)
pub fn find_crate(crate_name: &str, after_name: TokenStream) -> Option<TokenStream> {
    match proc_macro_crate::crate_name(crate_name) {
        Ok(FoundCrate::Itself) => Some(quote! {crate #after_name}),
        Ok(FoundCrate::Name(n)) => {
            let name = syn::Ident::new(&n, Span::call_site());
            Some(quote! {#name #after_name})
        }
        _ => None,
    }
}

/// Attempts to find any of multiple crates, returning the path for the first one found.
///
/// This function is useful when a procedural macro can work with multiple alternative
/// crates or when you want to provide fallback options. It tries each crate in the
/// provided list and returns the path for the first one that exists in Cargo.toml.
/// Like [`find_crate`], it properly handles crate renaming.
///
/// # Crate Renaming Support
///
/// Each crate in the list is checked with full renaming support. If a crate has been
/// renamed in Cargo.toml using the `package` directive, this function will return
/// the correct import name that should be used.
///
/// # Arguments
///
/// * `list` - A slice of tuples containing `(original_crate_name, after_name_suffix)`
///
/// # Returns
///
/// * `Some(TokenStream)` - Path to the first crate found, with its suffix
/// * `None` - If none of the crates are found in Cargo.toml
///
/// # Examples
///
/// ```rust
/// use easy_macros_helpers_macro_safe::find_crate_list;
/// use quote::quote;
///
/// let crates = &[
///     ("tokio", quote!(::runtime)),
///     ("async-std", quote!(::task)),
///     ("smol", quote!()),
/// ];
///
/// if let Some(async_runtime) = find_crate_list(crates) {
///     // Uses the first available async runtime crate
/// }
///
/// // With renamed crates in Cargo.toml:
/// // [dependencies]
/// // tokio_renamed = { package = "tokio", version = "1.0" }
/// // async_std = "1.0"
/// let crates = &[
///     ("tokio", quote!(::runtime)),      // Will find "tokio_renamed"
///     ("async-std", quote!(::task)),     // Will find "async_std"
/// ];
/// if let Some(path) = find_crate_list(crates) {
///     // Returns: tokio_renamed::runtime
/// }
/// ```
///
/// # Use Cases
///
/// - Supporting multiple versions of a crate with different names
/// - Providing fallback options for optional dependencies
/// - Working with crates that have been renamed or reorganized
/// - Creating flexible macros that adapt to available dependencies
/// - Handling ecosystem transitions (e.g., futures-preview → futures)
///
/// # Generated Output Examples
///
/// For input `[("serde", quote!(::Serialize)), ("serde_derive", quote!())]`:
/// - Returns `serde::Serialize` if `serde` is found (normal dependency)
/// - Returns `serde_renamed::Serialize` if `serde` was renamed to `serde_renamed`
/// - Returns `serde_derive` if only `serde_derive` is found
/// - Returns `None` if neither is found
pub fn find_crate_list(list: &[(&str, TokenStream)]) -> Option<TokenStream> {
    for (name, after_name) in list {
        if let Some(result) = find_crate(name, after_name.clone()) {
            return Some(result);
        }
    }
    None
}
