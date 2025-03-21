use all_syntax_cases::all_syntax_cases;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use super::FoundContextInfo;

struct ArgData {
    display_helper: TokenStream,
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
    let mut to_remove = None;
    for (index, attr) in attrs.iter_mut().enumerate() {
        if attr.path().is_ident("context") {
            if let syn::Meta::List(l) = &attr.meta {
                let tokens_str = l.tokens.to_string();
                let tokens_str_no_space = tokens_str.replace(|c: char| c.is_whitespace(), "");
                if tokens_str_no_space.as_str() == "path" {
                    data.display_helper = quote! {
                        .display()
                    };
                    to_remove = Some(index);
                }
            }
        }
    }
    if let Some(index) = to_remove {
        attrs.remove(index);
    }
}

pub fn arg_handle(arg: &mut syn::Expr, context_info: &mut FoundContextInfo) {
    let mut data = ArgData {
        display_helper: quote! {},
    };
    arg_expr_handle(arg, &mut data);

    let display_helper = data.display_helper;

    context_info
        .inputs_found
        .push(quote! {#arg #display_helper});
}
