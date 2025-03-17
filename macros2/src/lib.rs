mod always_context;
mod context_internal;
mod macro_result;

use proc_macro::TokenStream;

#[proc_macro_attribute]
///Adds .with_context(context!()) before all '?' without them
pub fn always_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    always_context::always_context(attr, item)
}

#[proc_macro]
///Use context! macro from helpers crate instead
pub fn context_internal(item: TokenStream) -> TokenStream {
    context_internal::context_internal(item)
}

#[proc_macro_attribute]
/// Allows for macros with `anyhow::Result<TokenStream>` return type
///
///Creates a wrapper for passed in function, passed in function is placed inside of wrapper
pub fn macro_result(attr: TokenStream, item: TokenStream) -> TokenStream {
    macro_result::macro_result(attr, item)
}
