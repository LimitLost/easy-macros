use helpers_macro_safe::indexed_name;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, punctuated::Punctuated};
///Format: `matched_check_no_fields!(match_path(struct_path,struct_path2,...))`
struct Input {
    match_path: syn::Path,
    _paren: syn::token::Paren,
    struct_paths: Punctuated<syn::Path, Token![,]>,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let match_path = input.parse()?;

        let insides1;
        let _paren = syn::parenthesized!(insides1 in input);
        let struct_paths = Punctuated::parse_terminated(&insides1)?;
        Ok(Input {
            match_path,
            _paren,
            struct_paths,
        })
    }
}

///Macro used by all_syntax_cases
///
/// Format: `matched_check_no_fields!(match_path(struct_path,struct_path2,...))`
///
/// Uses `result_matches`, `default_functions`, `system_functions` and `special_functions`, without requesting them in macro input
pub fn matched_check_no_fields(item: TokenStream) -> TokenStream {
    let Input {
        match_path,
        struct_paths,
        _paren: _,
    } = syn::parse_macro_input!(item as Input);

    let call_names = indexed_name(quote::format_ident!("struct_call"), struct_paths.len());

    let call_structs = struct_paths.iter();

    let result = quote! {
        {

            //Handle fields from macro input (only structs)
            let fields1: Vec<syn::Field> = vec![#(syn::parse_quote!{
                #call_names:&mut #call_structs
            }),*];

            let mut special_call = None;
            //Find matching special function, if any
            for func in special_functions.iter_mut(){
                if let Some(call) = func.all_inputs_check(&fields1, None, (additional_input_name, additional_input_ty)){
                    special_call = Some(call);
                    break;
                }
            }

            //Resulting match arm
            result_matches.extend(quote! {
                #match_path(#(#call_names),*)=>
            });
            //Result match block
            //Find matching default functions, if no special function was found
            if let Some(call) = special_call{
                //Workaround since we can't create #example without spaces between tokens (added by compiler)
                let call_braced = crate::helpers::braced(call.into_token_stream());

                result_matches.extend(call_braced);
            }else{
                let mut default_calls = Vec::new();
                //Functions provided by user
                for func in default_functions.iter_mut(){
                    if let Some(call) = func.all_inputs_check(&fields1, None, (additional_input_name, additional_input_ty)){
                        default_calls.push(call);
                    }
                }
                //Functions used by the macro, for example for search
                for func in system_functions.iter_mut(){
                    if let Some(call) = func.all_inputs_check(&fields1, None, (additional_input_name, additional_input_ty)){
                        default_calls.push(call);
                    }
                }

                // Workaround since we can't create #example without spaces between tokens (added by compiler)
                let default_calls_braced = crate::helpers::braced(crate::helpers::iter_token_stream(default_calls.into_iter()));

                result_matches.extend(default_calls_braced);
            }
        }
    };

    // panic!("Result: {}", result);

    result.into()
}
