mod context_gen;
mod search;
mod settings;
use settings::Settings;

use helpers::find_crate_list;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use search::item_handle;

fn crate_missing_panic(crate_name: &str, for_macro: &str) -> ! {
    panic!(
        "Using {for_macro} requires `{crate_name}` (or `easy-macros` crate) to be present in dependencies! You can add it with `{crate_name} = \"*\"` in your Cargo.toml dependencies or with `cargo add {crate_name}` command."
    );
}

fn context_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[
        ("easy-macros", quote! {}),
        ("easy-macros-helpers", quote! {}),
    ]) {
        found
    } else {
        crate_missing_panic("easy-macros-helpers", "always_context");
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
/// ## Macro-level
/// - Skip
///     - `easy_sql` or `es` - Skips requiring context for query! macros
///     - `skip_macros` or `!` - Skips requiring context for macros entirely
///      
/// Examples: `#[always_context(skip(!))]`, `#[always_context(skip(macros, easy_sql))]`, `#[always_context(skip(!, es))]`
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
/// - `#[context(ignore)]` or `#[context(ignored)]` or `#[context(no)]` - Exclude this argument from context
///
/// # Limitations
///
/// These expressions before `?` require manual `.with_context()` or `.context()`:
/// blocks, control flow (`if`/`match`/`while`/`for`/`loop`), field access, macros.
pub fn always_context(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as syn::Item);

    let settings = syn::parse_macro_input!(attr as Settings);

    item_handle(&mut parsed, settings);

    parsed.into_token_stream().into()
}
#[proc_macro_attribute]
/// Debug version of `always_context` that panics with the result.
#[doc(hidden)]
pub fn always_context_debug(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as syn::Item);

    let settings = syn::parse_macro_input!(attr as Settings);

    item_handle(&mut parsed, settings);

    let debug_tokens = parsed.into_token_stream().to_string();
    let error_tokens = quote! {
        compile_error!(#debug_tokens);
    };
    error_tokens.into()
}
