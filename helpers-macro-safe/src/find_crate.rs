use proc_macro_crate::FoundCrate;
use proc_macro2::{Span, TokenStream};
use quote::quote;

/// Tries to find crate in the current Cargo.toml (where macro using this is used)
///
/// # Return format examples
/// - with `after_name` empty
/// ```rust
///     crate
///     your_crate_name
/// ```
/// - with `after_name` set to `::help`
/// ```rust
///     crate::help
///     your_crate_name::help
/// ```
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

/// Tries one of the input crates in the current Cargo.toml (where macro using this is used)
///
/// # Arguments
/// `list` - `&[(crate_name: &str, after_name: TokenStream)]`
///
/// # Return format examples
/// - with `after_name` empty
/// ```rust
///     crate
///     your_crate_name
/// ```
/// - with `after_name` set to `::help`
/// ```rust
///     crate::help
///     your_crate_name::help
/// ```
pub fn find_crate_list(list: &[(&str, TokenStream)]) -> Option<TokenStream> {
    for (name, after_name) in list {
        if let Some(result) = find_crate(name, after_name.clone()) {
            return Some(result);
        }
    }
    None
}
