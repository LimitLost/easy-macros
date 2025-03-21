mod context_gen;
mod search;

use proc_macro::TokenStream;
use quote::ToTokens;
use search::item_handle;

#[proc_macro_attribute]
/// Procedural Macro Attribute
///
/// Adds .with_context(context!()) before all '?' without them
///
/// Also adds function names and inputs into context!() macro (by default)
///
/// # Supported attributes (general)
///
/// (for example: in the selected scope, before expression)
///
/// - `#[no_context]` - Don't put `.with_context(context!())` at all
/// - `#[no_context_inputs]` - Don't put function names and inputs in `context!()`, does just `.with_context(context!())`
/// - `#[enable_context]` - Enable context generation back, if it was disabled by other attribute, it's also auto disabled in macros
///
/// # Supported attributes (function call arguments)
///
/// - `#[context(path)]` - adds `.display()` after argument, inside of `context!()` macro, useful for `std::path::PathBuf` and `std::path::Path`
pub fn always_context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as syn::Item);
    //Adds .with_context(context!()) before all '?' without them
    //Maybe add also function inputs with names into context?

    item_handle(&mut parsed, None);

    parsed.into_token_stream().into()
}
