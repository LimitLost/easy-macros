use always_context::always_context;
use helpers::{TokensBuilder, find_crate, parse_macro_input};
use quote::quote;

use crate::{
    data::{HandleMaybeRefAttrsInput, Reference},
    root_macros_crate,
};

fn crate_missing_panic(crate_name: &str) -> ! {
    panic!(
        "Using fields_with_attributes requires `{crate_name}` crate to be present in dependencies! You can add it with `{crate_name} = \"*\"` in your Cargo.toml dependencies or with `cargo add {crate_name}` command."
    );
}
fn syn_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate("syn", quote! {}) {
        found
    } else {
        crate_missing_panic("syn");
    }
}

#[always_context]
pub fn fields_with_attributes(
    item: proc_macro::TokenStream,
) -> anyhow::Result<proc_macro::TokenStream> {
    let parsed = parse_macro_input!(item as HandleMaybeRefAttrsInput);

    let syn_crate = syn_crate();

    let operate_on = parsed.operate_on;
    let attributes = parsed.attributes;
    let mut result = TokensBuilder::default();

    let (iter, reference) = match parsed.reference {
        Some(Reference::Ref) => (quote! { .iter() }, quote! { & }),
        Some(Reference::RefMut) => (quote! { .iter_mut() }, quote! { &mut }),
        None => (quote! { .into_iter() }, quote! {}),
    };

    let crate_root = root_macros_crate("fields_with_attributes");

    result.add(quote! {
        {
            let fields=match #reference #operate_on.fields{
                #syn_crate::Fields::Named(fields) => {
                    Some(#reference fields.named)
                }
                #syn_crate::Fields::Unnamed(fields) => {
                    Some(#reference fields.unnamed)
                }
                #syn_crate::Fields::Unit => {
                    None
                }
            };

            fields
            .into_iter()
            .map(|f| {
                f #iter .enumerate() .filter_map(|(index,field)|{
                    if #crate_root::has_attributes!(field,#(#attributes)*) {
                        Some((index, field))
                    } else {
                        None
                    }
                })
            })
            .flatten()
        }
    });

    Ok(result.finalize().into())
}
