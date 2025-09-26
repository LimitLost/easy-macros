use super::data::HandleAttrsInput;
use always_context::always_context;
use easy_macros_helpers_macro_safe::{MacroResult, indexed_name, parse_macro_input};
use proc_macro::TokenStream;
use quote::quote;

#[always_context]
///Returns true if the passed in item has all passed in attributes (one or more)
pub fn has_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    let parsed = parse_macro_input!(item as HandleAttrsInput);

    let operate_on = parsed.operate_on;
    let attributes = parsed.attributes;
    let mut result = MacroResult::default();

    let attributes_len = attributes.len();

    let attr_to_find_vars = indexed_name(quote::format_ident!("attr_to_find"), attributes_len);
    let found_vars = indexed_name(quote::format_ident!("found_vars"), attributes_len);

    let mut maybe_break = quote! {};
    //Add break; if only one attribute is passed in
    if attributes_len == 1 {
        maybe_break = quote! { break; };
    }

    //Check if attribute is present

    result.add(quote! {
        {
            #(
                let #attr_to_find_vars:syn::Attribute = syn::parse_quote! {
                    #attributes
                };
                let mut #found_vars = false;
            )*
            for attr in #operate_on.attrs.iter() {
                #(
                    if attr == &#attr_to_find_vars {
                        #found_vars = true;
                        #maybe_break
                    }
                )*
            }
            let mut found=true;
            #(
                if !#found_vars {
                    found=false;
                }
            )*
            found
        }
    });

    Ok(result.finalize().into())
}
