use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, punctuated::Punctuated, spanned::Spanned};

///Format: `matched_check!(match_path(struct_path{fields}))`
struct Input {
    match_path: syn::Path,
    _paren: syn::token::Paren,
    struct_path: syn::Path,
    _brace: syn::token::Brace,
    fields: Punctuated<syn::Field, Token![,]>,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let match_path = input.parse()?;

        let insides1;
        let _paren = syn::parenthesized!(insides1 in input);
        let struct_path = insides1.parse()?;

        let fields_named: syn::FieldsNamed = insides1.parse()?;
        let _brace = fields_named.brace_token;
        let fields = fields_named.named;
        Ok(Input {
            match_path,
            _paren,
            struct_path,
            _brace,
            fields,
        })
    }
}

///Macro used by all_syntax_cases
///
/// Format: `matched_check!(match_path(struct_path{fields}))`
pub fn matched_check(item: TokenStream) -> TokenStream {
    let Input {
        match_path,
        struct_path,
        fields,
        _paren: _,
        _brace: _,
    } = syn::parse_macro_input!(item as Input);

    let fields_check = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_ty = &field.ty;

        let value_quote = quote::quote_spanned! {field_ty.span()=>
            crate::helpers::never_any::<#field_ty>()
        };
        quote! {
            #field_name: #value_quote,
        }
    });

    let fields_vec = fields.iter();

    let struct_call_name = quote::format_ident!("a");

    // Supports only one match argument for now
    // Uses result_matches default_functions special_functions
    let result = quote! {
        {
            //check if fields are valid
            let _=||{
                let _= #struct_path {
                    #(#fields_check)*
                };
            };
            //struct call token stream
            let struct_call = quote! { #struct_call_name };

            //Handle fields from macro input (only struct)
            let fields1: Vec<syn::Field> = vec![syn::parse_quote!{
                #struct_call_name:#struct_path
            }];
            //Handle fields from macro input (inside of struct)
            let fields2: Vec<syn::Field> = vec![#(syn::parse_quote!{
                #fields_vec
            }),*];

            let mut special_call = None;
            //Find matching special function, if any
            for func in special_functions.iter_mut(){
                if let Some(call) = func.all_inputs_check(&fields1, None, (&additional_input_name, additional_input_ty)){
                    special_call = Some(call);
                    break;
                }
                if let Some(call) = func.all_inputs_check(&fields2, Some(&struct_call), (&additional_input_name, additional_input_ty)){
                    special_call = Some(call);
                    break;
                }
            }

            //Find matching default functions, if no special function was found
            if let Some(call) = special_call{
                //Workaround since we can't create #example without spaces between tokens (added by compiler)
                let call_braced = crate::helpers::braced(call.into_token_stream());

                //Resulting match arm
                result_matches.extend(quote! {
                    #match_path(#struct_call_name)=>
                });
                result_matches.extend(call_braced);
            }else{
                let mut default_calls= Vec::new();

                for func in default_functions.iter_mut(){
                    if let Some(call) = func.all_inputs_check(&fields1, None, (&additional_input_name, additional_input_ty)){
                        default_calls.push(call);
                    }
                    if let Some(call) = func.all_inputs_check(&fields2, Some(&struct_call), (&additional_input_name, additional_input_ty)){
                        default_calls.push(call);
                    }
                }

                // Workaround since we can't create #example without spaces between tokens (added by compiler)
                let default_calls_braced = crate::helpers::braced(crate::helpers::iter_token_stream(default_calls.into_iter()));

                // Resulting match arm
                result_matches.extend(quote! {
                    #match_path(#struct_call_name) =>
                });
                result_matches.extend(default_calls_braced);
            }
        }
    };

    // panic!("Result: {}", result);

    result.into()
}
