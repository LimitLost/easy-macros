use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, punctuated::Punctuated, spanned::Spanned};
///Format: `matched_check_no_fields!(match_path(struct_path,struct_path2,...))`
struct Input {
    struct_path: syn::Path,
    _brace: syn::token::Brace,
    fields: Punctuated<syn::Field, Token![,]>,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let struct_path = input.parse()?;

        let fields_named: syn::FieldsNamed = input.parse()?;
        let _brace = fields_named.brace_token;
        let fields = fields_named.named;
        Ok(Input {
            struct_path,
            _brace,
            fields,
        })
    }
}

///Macro used by all_syntax_cases
///
/// Format: `matched_check!(match_path(struct_path{fields}))`
///
/// Uses `result`, `default_functions`, `system_functions` and `special_functions`, without requesting them in macro input
pub fn struct_check(item: TokenStream) -> TokenStream {
    let Input {
        struct_path,
        fields,
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

    let struct_call_name = quote::format_ident!("search_item");

    // Supports only one match argument for now
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
                #struct_call_name:&mut #struct_path
            }];
            //Handle fields from macro input (inside of struct)
            let fields2: Vec<syn::Field> = vec![#(syn::parse_quote!{
                #fields_vec
            }),*];

            let mut special_call = None;
            //Find matching special function, if any
            for func in special_functions.iter_mut(){
                if let Some(call) = func.all_inputs_check(&fields1, None, (additional_input_name, additional_input_ty)){
                    special_call = Some(call);
                    break;
                }
                if let Some(call) = func.all_inputs_check(&fields2, Some(&struct_call), (additional_input_name, additional_input_ty)){
                    special_call = Some(call);
                    break;
                }
            }


            //Resulting function calls
            //Find matching default functions, if no special function was found
            if let Some(call) = special_call{
                result.extend(call.into_token_stream());
            }else{
                let mut default_calls= Vec::new();
                //Functions provided by user
                for func in default_functions.iter_mut(){
                    if let Some(call) = func.all_inputs_check(&fields2, Some(&struct_call), (additional_input_name, additional_input_ty)){
                        default_calls.push(call);
                    }
                }
                //Functions used by the macro, for example for search
                for func in system_functions.iter_mut(){
                    if let Some(call) = func.all_inputs_check(&fields2, Some(&struct_call), (additional_input_name, additional_input_ty)){
                        default_calls.push(call);
                    }
                }
                //Functions provided by user with #[after_system]
                for func in default_functions_after_system.iter_mut(){
                    if let Some(call) = func.all_inputs_check(&fields2, Some(&struct_call), (additional_input_name, additional_input_ty)){
                        default_calls.push(call);
                    }
                }


                result.extend(crate::helpers::iter_token_stream(default_calls.into_iter()));
            }
        }
    };

    // panic!("Result: {}", result);

    result.into()
}
