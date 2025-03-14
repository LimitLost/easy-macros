use helpers::{MacroResult, indexed_name};
use macros2::macro_result;
use proc_macro::TokenStream;
use quote::quote;

struct HasAttributeInput {
    operate_on: syn::Expr,
    _comma: syn::token::Comma,
    attributes: Vec<syn::Attribute>,
}

impl syn::parse::Parse for HasAttributeInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let operate_on = input.parse()?;
        let _comma = input.parse()?;
        let attributes = syn::Attribute::parse_outer(input)?;

        Ok(HasAttributeInput {
            operate_on,
            _comma,
            attributes,
        })
    }
}

#[proc_macro]
#[macro_result]
///Returns true if the passed in item has all passed in attributes (one or more)
pub fn has_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    let parsed = helpers::parse_macro_input!(item as HasAttributeInput);

    let operate_on = parsed.operate_on;
    let attributes = parsed.attributes;
    let mut result = MacroResult::new();

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

//TODO Allow for only one unknown inside of attribute
// __unknown__ - unknown mark
//Example: #[attribute__unknown__]
//Example: #[attri__unknown__bute]
//Example: #[__unknown__attribute]
//Example: #[attribute(__unknown__)]
//Example: #[attribute(name=__unknown__)]
//Example: #[attribute = __unknown__]
#[proc_macro]
#[macro_result]
pub fn get_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    //The easiest way would be just turning attributes into a string and then parsing it
    //We would have to parse some parts into string anyway and this isn't performance critical
}
