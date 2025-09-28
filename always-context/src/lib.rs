mod context_gen;
mod search;

use helpers::find_crate_list;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use search::item_handle;

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

#[proc_macro_attribute]
/// Procedural Macro Attribute
///
/// Adds .with_context(context!()) before all '?' without them (expects anyhow::Result function output, don't use type aliases)
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
/// - `#[context(display)]` - uses `{}` instead of `{:?}`, inside of `context!()` macro, useful when our input doesn't have `Debug` trait but it has `Display` trait
/// - `#[context(.example())` - uses `.example()` on our argument to get value for display, inside of `context!()` macro
/// - `#[context(tokens)]` -  same as `#[context(display)] #[context(.to_token_stream())]`
/// - `#[context(tokens_vec)]` - same as `#[context(display)] #[context(.iter().map(|el|el.to_token_stream()).collect::<TokenStream>())]`
/// - `#[context(not_sql)]` - use on `sql!` and `sql_where!` macros if they are not a part of `easy_sql`
pub fn always_context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as syn::Item);
    //Adds .with_context(context!()) before all '?' without them
    //Maybe add also function inputs with names into context?

    item_handle(&mut parsed, None);

    parsed.into_token_stream().into()
}
