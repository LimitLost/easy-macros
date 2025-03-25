mod data;
mod fields_get_attributes;
mod fields_with_attributes;
mod get_attributes;
mod has_attributes;

use always_context::always_context;
use helpers_macro_safe::find_crate_list;
use macro_result::macro_result;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};

fn root_macros_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[
        ("easy-lib", quote! {::macros}),
        ("easy-macros", quote! {::macros}),
        ("attributes", quote! {}),
        ("attributes-macros", quote! {}),
    ]) {
        found
    } else {
        quote! {self}
    }
}

#[always_context]
#[proc_macro]
#[macro_result]
///Returns true if the passed in item has all passed in attributes (one or more)
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
#[macro_result]
///# Examples
/// ```rust
/// // item must have `.attrs` field
/// get_attributes!(item, #[attribute__unknown__]);
/// // All attributes without unknown (in this example #[attr1]) must be present in item to get anything other than empty vector
/// get_attributes!(item, #[attr1]#[attri__unknown__bute]);
/// get_attributes!(item, #[attr1]#[attribute(__unknown__)] #[attr2]);
/// //There can only be one unknown in our attribute list (for now at least)
/// ```
///# Return
/// Returns a `Vec<proc_macro2::TokenStream>` of unknown replacements found in the passed in item
pub fn get_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    get_attributes::get_attributes(item)
}

#[always_context]
#[proc_macro]
#[macro_result]
///# Examples
/// ```rust
/// // item must have `.fields` field
/// fields_with_attributes!(item, #[attribute]);
/// // All passed in attributes must be present on item field to be returned
/// fields_with_attributes!(item, #[attr1]#[attr2]);
/// // fields from the item will be taken with `.into_iter()`
/// fields_with_attributes!(item, #[attr1]#[attr2]#[attr3]);
/// // Use `&` to use `.iter()` instead
/// fields_with_attributes!(&item, #[attr1]#[attr2]#[attr3] );
/// // Use `&mut` to use `.iter_mut()` instead
/// fields_with_attributes!(&mut item, #[attr1]#[attr2]#[attr3]);
/// ```
/// # Return
/// Returns `Iterator<Item = syn::Field>` of fields with the passed in attributes (all attributes must be present)
pub fn fields_with_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    fields_with_attributes::fields_with_attributes(item)
}

#[always_context]
#[proc_macro]
#[macro_result]
///# Examples
/// ```rust
/// // item must have `.fields` field
/// fields_get_attributes!(item, #[attribute__unknown__]);
/// // All passed in attributes must be present on item field to be returned
/// fields_get_attributes!(item, #[attr1]#[__unknown__attr2]);
/// // fields from the item will be taken with `.into_iter()`
/// fields_get_attributes!(item, #[attr1]#[attr2]#[attr3(__unknown__)]);
/// // // Use `&` to use `.iter()` instead
/// fields_get_attributes!(&item, #[attr1]#[attr2]#[attr3(__unknown__)]);
/// // // Use `&mut` to use `.iter_mut()` instead
/// fields_get_attributes!(&mut item, #[attr1]#[attr2]#[attr3(__unknown__)]);
/// ```
/// # Return
/// Returns `Iterator<Item = (syn::Field, Vec<proc_macro2::TokenStream>) >` where `.0` is a field and `.1` are unknown replacements, all attributes must be present, for the field to be returned
pub fn fields_get_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    fields_get_attributes::fields_get_attributes(item)
}
