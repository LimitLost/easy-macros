use all_syntax_cases::all_syntax_cases;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::spanned::Spanned;

use super::{FoundContextInfo, InputFound};

struct ArgData {
    display_fn_call: TokenStream,
    display: bool,
}

all_syntax_cases! {
    setup=>{
        generated_fn_prefix: "arg",
        additional_input_type: &mut ArgData
    }
    default_cases=>{}
    special_cases=>{
        fn handle_arg_attrs(attrs: &mut Vec<syn::Attribute>, data: &mut ArgData);
    }
}

///Clears all always-context attributes
fn handle_arg_attrs(attrs: &mut Vec<syn::Attribute>, data: &mut ArgData) {
    let mut to_remove = Vec::new();
    for (index, attr) in attrs.iter_mut().enumerate() {
        if attr.path().is_ident("context") {
            if let syn::Meta::List(l) = &attr.meta {
                let tokens = &l.tokens;
                let tokens_str = tokens.to_string();
                let tokens_str_no_space = tokens_str.replace(|c: char| c.is_whitespace(), "");
                match tokens_str_no_space.as_str() {
                    "display" => {
                        data.display = true;
                        to_remove.push(index);
                    }
                    "tokens" => {
                        data.display = true;
                        let tokens_span = tokens.span();
                        data.display_fn_call = quote_spanned! {tokens_span=> .to_token_stream() };
                        to_remove.push(index);
                    }
                    "tokens_vec" => {
                        data.display = true;
                        let tokens_span = tokens.span();
                        data.display_fn_call = quote_spanned! {tokens_span=> .iter().map(|el|el.to_token_stream()).collect::<TokenStream>() };
                        to_remove.push(index);
                    }
                    _ => {
                        if tokens_str_no_space.starts_with(".") {
                            data.display_fn_call = tokens.clone();
                            to_remove.push(index);
                        }
                    }
                }
            }
        }
    }
    for index in to_remove.iter().rev() {
        attrs.remove(*index);
    }
}

pub fn arg_handle(arg: &mut syn::Expr, context_info: &mut FoundContextInfo) {
    let mut data = ArgData {
        display_fn_call: quote! {},
        display: false,
    };
    arg_expr_handle(arg, &mut data);

    let display_fn_call = data.display_fn_call;

    context_info.inputs_found.push(InputFound {
        input: quote! {(#arg) #display_fn_call},
        display: data.display,
    });
}
