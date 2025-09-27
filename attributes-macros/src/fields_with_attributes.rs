use always_context::always_context;
use easy_macros_helpers_macro_safe::{TokensBuilder, parse_macro_input};
use quote::quote;

use crate::{
    data::{HandleMaybeRefAttrsInput, Reference},
    root_macros_crate,
};

#[always_context]
pub fn fields_with_attributes(
    item: proc_macro::TokenStream,
) -> anyhow::Result<proc_macro::TokenStream> {
    let parsed = parse_macro_input!(item as HandleMaybeRefAttrsInput);

    let operate_on = parsed.operate_on;
    let attributes = parsed.attributes;
    let mut result = TokensBuilder::default();

    let (iter, maybe_fields, unit_handle) = match parsed.reference {
        Some(Reference::Ref) => (
            quote! { .iter() },
            quote! { let f= syn::punctuated::Punctuated::new();},
            quote! { &f },
        ),
        Some(Reference::RefMut) => (
            quote! { .iter_mut() },
            quote! { let f= syn::punctuated::Punctuated::new();},
            quote! { &mut f },
        ),
        None => (
            quote! { .into_iter() },
            quote! {},
            quote! {Default::default()},
        ),
    };

    let crate_root = root_macros_crate();

    result.add(quote! {
        {
            #maybe_fields
            let fields=match #operate_on.fields{
                syn::Fields::Named(fields) => {
                    fields
                }
                syn::Fields::Unnamed(fields) => {
                    fields
                }
                syn::Fields::Unit => {
                    #unit_handle
                }
            };

            fields #iter .enumerate() .filter_map(|(index,field)|{
                if #crate_root::has_attributes!(field,#(#attributes)*) {
                    Some((index, field))
                } else {
                    None
                }
            })
        }
    });

    Ok(result.finalize().into())
}
