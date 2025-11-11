use always_context::always_context;
use helpers::{TokensBuilder, find_crate, parse_macro_input};
use quote::quote;

use crate::{
    context_crate,
    data::{HandleMaybeRefAttrsInput, Reference},
    root_macros_crate,
};

fn crate_missing_panic(crate_name: &str) -> ! {
    panic!(
        "Using fields_get_attributes requires `{crate_name}` crate to be present in dependencies! You can add it with `{crate_name} = \"*\"` in your Cargo.toml dependencies or with `cargo add {crate_name}` command."
    );
}
fn syn_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate("syn", quote! {}) {
        found
    } else {
        crate_missing_panic("syn");
    }
}
fn quote_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate("quote", quote! {}) {
        found
    } else {
        crate_missing_panic("quote");
    }
}

fn proc_macro2_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate("proc-macro2", quote! {}) {
        found
    } else {
        crate_missing_panic("proc-macro2");
    }
}

fn anyhow_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate("anyhow", quote! {}) {
        found
    } else {
        crate_missing_panic("anyhow");
    }
}

#[always_context]
pub fn fields_get_attributes(
    item: proc_macro::TokenStream,
) -> anyhow::Result<proc_macro::TokenStream> {
    let parsed = parse_macro_input!(item as HandleMaybeRefAttrsInput);

    let operate_on = parsed.operate_on;
    let attributes = parsed.attributes;
    let mut result = TokensBuilder::default();

    let syn_crate = syn_crate();
    let quote_crate = quote_crate();
    let proc_macro2_crate = proc_macro2_crate();
    let anyhow_crate = anyhow_crate();

    let (iter, ref_state) = match parsed.reference {
        Some(Reference::Ref) => (quote! { .iter() }, quote! {&}),
        Some(Reference::RefMut) => (quote! { .iter_mut() }, quote! {&mut}),
        None => (quote! { .into_iter() }, quote! {}),
    };

    let crate_root = root_macros_crate("fields_get_attributes");
    let context_crate = context_crate("fields_get_attributes");

    result.add(quote! {
        {
            use #quote_crate::ToTokens as _;
            let fields=match #ref_state #operate_on.fields{
                #syn_crate::Fields::Named(fields) => {
                    Some(fields.named #iter)
                }
                #syn_crate::Fields::Unnamed(fields) => {
                    Some(fields.unnamed #iter)
                }
                #syn_crate::Fields::Unit => {
                    None
                }
            };

            let mut errors: Vec<(#anyhow_crate::Result<()>, #ref_state #syn_crate::Field)> = Vec::new();

            let filtered: Vec<(usize,#ref_state #syn_crate::Field, Vec<#proc_macro2_crate::TokenStream>)> = fields.into_iter().flatten() .enumerate().filter_map(|(index, field)|{
                fn get_attrs(field:& #syn_crate::Field)->#anyhow_crate::Result<Vec<#proc_macro2_crate::TokenStream>>{
                    Ok(#crate_root::get_attributes!(field,#(#attributes)*))
                }

                let unknowns=get_attrs(&field);
                match unknowns{
                    Ok(unknowns)=>{
                        if unknowns.is_empty(){
                            None
                        }else {
                            Some((index,field,unknowns))
                        }
                    }
                    Err(err)=>{
                        errors.push((#anyhow_crate::Result::Err(err),field));
                        None
                    }
                }
            }).collect();

            for (err,field) in errors.into_iter(){
                err.with_context(#context_crate::context!("fields_get_attributes macro | field: {}",field.to_token_stream()))?;
            }

            filtered
        }
    });

    // panic!("{}", result.finalize());

    Ok(result.finalize().into())
}
