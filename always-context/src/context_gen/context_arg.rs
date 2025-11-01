use all_syntax_cases::all_syntax_cases;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::spanned::Spanned;

use super::{FoundContextInfo, InputFound};

struct ArgData {
    display_fn_call: TokenStream,
    display: bool,
    #[cfg(feature = "easy_sql")]
    not_sql: bool,
    ///"Are we working with duplicate used for generating context?"
    duplicate: bool,
    /// Ignore argument
    ignore: bool,
}

all_syntax_cases! {
    setup=>{
        generated_fn_prefix: "arg",
        additional_input_type: &mut ArgData
    }
    default_cases=>{}
    special_cases=>{
        fn macro_handle(macro_: &mut syn::ExprMacro, data: &mut ArgData);
        fn handle_arg_attrs(attrs: &mut Vec<syn::Attribute>, data: &mut ArgData);
    }
}

///Easy Sql integration
fn macro_handle(macro_: &mut syn::ExprMacro, data: &mut ArgData) {
    handle_arg_attrs(&mut macro_.attrs, data);

    //Perform this only on duplicate used for generating context
    if !data.duplicate {
        return;
    }

    //Add `debug_info_mode` keyword to the start of the sql! and query! macros
    #[cfg(feature = "easy_sql")]
    if !data.not_sql {
        let last_segment = if let Some(s) = macro_.mac.path.segments.last() {
            s
        } else {
            return;
        };
        let last_segment_str = last_segment.ident.to_string();
        if last_segment_str != "sql" && last_segment_str != "query" {
            return;
        }

        replace_with::replace_with_or_abort(
            &mut macro_.mac.tokens,
            |tokens| quote! { debug_info_mode #tokens},
        );
    }
}

///Clears all always-context attributes
fn handle_arg_attrs(attrs: &mut Vec<syn::Attribute>, data: &mut ArgData) {
    let mut to_remove = Vec::new();
    for (index, attr) in attrs.iter_mut().enumerate() {
        if attr.path().is_ident("context")
            && let syn::Meta::List(l) = &attr.meta
        {
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
                #[cfg(feature = "easy_sql")]
                "not_sql" => {
                    data.not_sql = true;
                    to_remove.push(index);
                }
                "ignore" | "ignored" | "no" => {
                    data.ignore = true;
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
    for index in to_remove.iter().rev() {
        attrs.remove(*index);
    }
}

pub fn arg_handle(arg: &mut syn::Expr, context_info: &mut FoundContextInfo) {
    let mut data = ArgData {
        display_fn_call: quote! {},
        display: false,
        #[cfg(feature = "easy_sql")]
        not_sql: false,
        duplicate: false,
        ignore: false,
    };

    let mut arg_cloned = arg.clone();

    arg_expr_handle(arg, &mut data);
    data.duplicate = true;
    arg_expr_handle(&mut arg_cloned, &mut data);

    if data.ignore {
        context_info.inputs_found.push(InputFound {
            input: quote! {"ignored"},
            display: true,
        })
    } else {
        let display_fn_call = data.display_fn_call;

        context_info.inputs_found.push(InputFound {
            input: quote! {(#arg_cloned) #display_fn_call},
            display: data.display,
        })
    };
}
