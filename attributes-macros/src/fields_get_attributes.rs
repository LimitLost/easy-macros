use always_context::always_context;
use easy_macros_helpers_macro_safe::{TokensBuilder, parse_macro_input};
use quote::quote;

use crate::{
    data::{HandleMaybeRefAttrsInput, Reference},
    root_macros_crate,
};

#[always_context]
pub fn fields_get_attributes(
    item: proc_macro::TokenStream,
) -> anyhow::Result<proc_macro::TokenStream> {
    let parsed = parse_macro_input!(item as HandleMaybeRefAttrsInput);

    let operate_on = parsed.operate_on;
    let attributes = parsed.attributes;
    let mut result = TokensBuilder::default();

    let (iter, maybe_fields, unit_handle, ref_state) = match parsed.reference {
        Some(Reference::Ref) => (
            quote! { .iter() },
            quote! { let f = syn::punctuated::Punctuated::new(); },
            quote! { &f },
            quote! {&},
        ),
        Some(Reference::RefMut) => (
            quote! { .iter_mut() },
            quote! { let f = syn::punctuated::Punctuated::new();},
            quote! { &mut f },
            quote! {&mut},
        ),
        None => (
            quote! { .into_iter() },
            quote! {},
            quote! {Default::default()},
            quote! {},
        ),
    };

    let crate_root = root_macros_crate();

    result.add(quote! {
        {
            #maybe_fields
            let fields=match #operate_on.fields{
                syn::Fields::Named(fields) => {
                    fields.named
                }
                syn::Fields::Unnamed(fields) => {
                    fields.unnamed
                }
                syn::Fields::Unit => {
                    #unit_handle
                }
            };

            let mut errors: Vec<(anyhow::Result<()>, #ref_state syn::Field)> = Vec::new();

            let filtered: Vec<(usize,#ref_state syn::Field, Vec<proc_macro2::TokenStream>)> = fields #iter .enumerate().filter_map(|(index, field)|{
                fn get_attrs(field:&syn::Field)->anyhow::Result<Vec<proc_macro2::TokenStream>>{
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
                        errors.push((anyhow::Result::Err(err),field));
                        None
                    }
                }
            }).collect();

            for (err,field) in errors.into_iter(){
                err.with_context(context!("fields_get_attributes macro | field: {}",field.to_token_stream()))?;
            }

            filtered
        }
    });

    // panic!("{}", result.finalize());

    Ok(result.finalize().into())
}
