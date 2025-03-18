mod always_context;

use proc_macro::TokenStream;

#[proc_macro_attribute]
///Adds .with_context(context!()) before all '?' without them
pub fn always_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    always_context::always_context(attr, item)
}
