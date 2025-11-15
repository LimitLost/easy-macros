mod context_gen;
mod search;

use helpers::find_crate_list;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use search::item_handle;

fn context_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[
        ("easy-macros", quote! {::helpers}),
        ("easy-macros-helpers", quote! {}),
    ]) {
        found
    } else {
        quote! {self}
    }
}

#[proc_macro_attribute]
/// Automatically adds `.with_context(context!())` to all `?` operators that don't already have context.
///
/// Transforms `operation()?` into `operation().with_context(context!("operation()"))?`
/// with function call details, arguments, and file location.
///
/// # Requirements
///
/// - Function must return `anyhow::Result<T>` or `Result<T, UserFriendlyError>` (please add an issue if you need support for other types)
///
/// # Control Attributes
///
/// ## Function-level
/// - `#[no_context]` - Disable context generation entirely
/// - `#[no_context_inputs]` - Add context but exclude function arguments  
/// - `#[enable_context]` - Re-enable context (useful in macros where auto-disabled)
///
/// ## Argument-level
/// - `#[context(display)]` - Use `Display` instead of `Debug` for formatting
/// - `#[context(.method())]` - Call method on argument before displaying
/// - `#[context(tokens)]` - Format as token stream (equivalent to `display` + `.to_token_stream()`)
/// - `#[context(tokens_vec)]` - Format as token stream collection
/// - `#[context(not_sql)]` - Use on `sql!` and `query!` macros if not part of `easy_sql` (requires `easy-sql` feature)
/// - `#[context(ignore)]` or `#[context(ignored)]` or `#[context(no)]` - Exclude this argument from context
///
/// # Limitations
///
/// These expressions before `?` require manual `.with_context()` or `.context()`:
/// blocks, control flow (`if`/`match`/`while`/`for`/`loop`), field access, macros.
pub fn always_context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as syn::Item);
    //Adds .with_context(context!()) before all '?' without them
    //Maybe add also function inputs with names into context?

    item_handle(&mut parsed, None);

    parsed.into_token_stream().into()
}
