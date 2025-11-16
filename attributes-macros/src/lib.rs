mod data;
mod fields_get_attributes;
mod fields_with_attributes;
mod get_attributes;
mod has_attributes;

use always_context::always_context;
use anyhow_result::anyhow_result;
use helpers::find_crate_list;
use proc_macro::TokenStream;
use quote::quote;

fn crate_missing_panic(crate_name: &str, for_macro: &str) -> ! {
    panic!(
        "Using {for_macro} requires `{crate_name}` (or `easy-macros` crate) to be present in dependencies! You can add it with `{crate_name} = \"*\"` in your Cargo.toml dependencies or with `cargo add {crate_name}` command."
    );
}

fn root_macros_crate(required_by: &str) -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[
        ("easy-macros", quote! {}),
        ("easy-macros-attributes", quote! {}),
        ("easy-macros-attributes-macros", quote! {}),
    ]) {
        found
    } else {
        crate_missing_panic("easy-macros-attributes", required_by);
    }
}

fn context_crate(required_by: &str) -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[
        ("easy-macros", quote! {}),
        ("easy-macros-helpers", quote! {}),
        ("easy-macros-attributes", quote! {::helpers}),
    ]) {
        found
    } else {
        crate_missing_panic("easy-macros-helpers", required_by);
    }
}

#[always_context]
#[proc_macro]
#[anyhow_result]
pub fn has_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    has_attributes::has_attributes(item)
}

// fn find_unknown(attr_template:&syn::Attribute,attr:syn::)

//Allow for only one unknown inside of attribute
// __unknown__ - unknown mark
//Example: #[attribute__unknown__]
//Example: #[attri__unknown__bute]
//Example: #[__unknown__attribute]
//Example: #[attribute(__unknown__)]
//Example: #[attribute(name=__unknown__)]
//Example: #[attribute = __unknown__]

#[always_context]
#[proc_macro]
#[anyhow_result]
pub fn get_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    get_attributes::get_attributes(item)
}

#[always_context]
#[proc_macro]
#[anyhow_result]
pub fn fields_with_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    fields_with_attributes::fields_with_attributes(item)
}

#[always_context]
#[no_context]
#[proc_macro]
#[anyhow_result]
/// Debug version of `fields_with_attributes!` that panics with the result.
#[doc(hidden)]
pub fn fields_with_attributes_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let result = fields_with_attributes::fields_with_attributes(item)?;
    panic!("{result}",);
}

#[always_context]
#[proc_macro]
#[anyhow_result]
pub fn fields_get_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    fields_get_attributes::fields_get_attributes(item)
}

#[always_context]
#[no_context]
#[proc_macro]
#[anyhow_result]
/// Debug version of `fields_get_attributes!` that panics with the result.
#[doc(hidden)]
pub fn fields_get_attributes_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let result = fields_get_attributes::fields_get_attributes(item)?;
    panic!("{result}",);
}
