use easy_macros_helpers_macro_safe::find_crate_list;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};

fn external_crate_parent() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[("easy-lib", quote! {}), ("easy-macros", quote! {})]) {
        found
    } else {
        quote! {}
    }
}

#[proc_macro_attribute]
/// Allows for macros with `anyhow::Result<TokenStream>` return type
///
///Creates a wrapper for passed in function, passed in function is placed inside of wrapper
pub fn macro_result(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut our_func = syn::parse_macro_input!(item as syn::ItemFn);

    let parent_crate = external_crate_parent();

    //Check if output of our function is a anyhow::Result<TokenStream>
    let func_output = &our_func.sig.output;
    match func_output {
        syn::ReturnType::Default => {
            panic!("Function must return a anyhow::Result<TokenStream>")
        }
        syn::ReturnType::Type(_, ty) => {
            let ty_str = ty
                .to_token_stream()
                .to_string()
                .replace(|c: char| c.is_whitespace(), "");
            if ty_str != "anyhow::Result<TokenStream>"
                && ty_str != "anyhow::Result<proc_macro::TokenStream>"
            {
                panic!("Function must return a anyhow::Result<TokenStream>");
            }
        }
    }

    let func_name = &our_func.sig.ident;

    let inputs = &our_func.sig.inputs;
    //inputs as arguments to function call
    let inputs_passed_in = inputs.iter().enumerate().map(|(index, arg)| match arg {
        syn::FnArg::Typed(arg) => {
            let pat = &arg.pat;
            if index == 1 {
                //Clone our item
                quote::quote! {
                    #pat.clone(),
                }
            } else {
                quote::quote! {
                    #pat,
                }
            }
        }
        _ => panic!("Self arguments shouldn't be supported on procedural macros"),
    });

    //If our function has #[proc_macro] attribute return nothing on error
    //If our function has #[proc_macro_derive] attribute return nothing on error
    //If our function has #[proc_macro_attribute] attribute return back item (second argument) on error
    //If our function has neither of those attributes panic
    let (err_result, macro_attr, attr_index) = {
        let mut err_result = None;
        let mut macro_attr = None;
        let mut attr_index = None;
        for (index, attr) in our_func.attrs.iter().enumerate() {
            let attr_name = attr.path().to_token_stream().to_string();
            if attr_name == "proc_macro" || attr_name == "proc_macro_derive" {
                err_result = Some(quote::quote! {
                let formatted_error = format!("{:?}", ___macro_err);
                let mut result=#parent_crate::quote::quote! {compile_error!};

                //Adds (formatted_error) to the end of the result
                result.extend( #parent_crate::proc_macro2::TokenStream::from(#parent_crate::proc_macro2::TokenTree::Group(#parent_crate::proc_macro2::Group::new(
                    #parent_crate::proc_macro2::Delimiter::Parenthesis,
                    #parent_crate::syn::LitStr::new(&formatted_error, #parent_crate::proc_macro2::Span::call_site()).into_token_stream(),
                ))));

                result.extend(#parent_crate::quote::quote! {;});

                result });
                macro_attr = Some(attr.clone());
                attr_index = Some(index);
                break;
            } else if attr_name == "proc_macro_attribute" {
                let second_input_arg = if let Some(arg) = inputs.iter().nth(1) {
                    if let syn::FnArg::Typed(arg) = arg {
                        arg.pat.clone()
                    } else {
                        panic!("Expected a typed argument");
                    }
                } else {
                    panic!("proc_macro_attribute function must have two arguments");
                };
                err_result = Some(quote::quote! {
                    let formatted_error= format!("{:?}", ___macro_err);
                    let mut result = #parent_crate::quote::quote! {compile_error!};

                    //Adds (formatted_error) to the end of the result
                    result.extend( #parent_crate::proc_macro2::TokenStream::from(#parent_crate::proc_macro2::TokenTree::Group(#parent_crate::proc_macro2::Group::new(
                        #parent_crate::proc_macro2::Delimiter::Parenthesis,
                        #parent_crate::syn::LitStr::new(&formatted_error, #parent_crate::proc_macro2::Span::call_site()).into_token_stream(),
                    ))));

                    result.extend(#parent_crate::quote::quote! {;});

                    result.extend(#parent_crate::proc_macro2::TokenStream::from(#second_input_arg));
                    result
                });
                macro_attr = Some(attr.clone());
                attr_index = Some(index);
                break;
            }
        }
        match (err_result, macro_attr, attr_index) {
            (Some(err_result), Some(macro_attr), Some(attr_index)) => {
                (err_result, macro_attr, attr_index)
            }
            _ => panic!(
                "Function must have either #[proc_macro] or #[proc_macro_derive] or #[proc_macro_attribute] attribute!"
            ),
        }
    };

    our_func.attrs.remove(attr_index);

    let result = quote::quote! {
        #macro_attr
        pub fn #func_name(#inputs) -> TokenStream {

            use #parent_crate::quote::ToTokens;

            #our_func

            match #func_name(#(#inputs_passed_in)*) {
                Ok(value) => value,
                Err(___macro_err) => {#err_result .into()},
            }
        }
    };

    // panic!("{}", result.to_string());

    result.into()
}
