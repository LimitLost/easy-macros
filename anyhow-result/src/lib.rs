use helpers::find_crate;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};

fn crate_missing_panic(crate_name: &str) -> ! {
    panic!(
        "Using anyhow-result requires `{crate_name}` crate to be present in dependencies! You can add it with `{crate_name} = \"*\"` in your Cargo.toml dependencies or with `cargo add {crate_name}` command."
    );
}

fn quote_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate("quote", quote! {}) {
        found
    } else {
        crate_missing_panic("quote");
    }
}

fn syn_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate("syn", quote! {}) {
        found
    } else {
        crate_missing_panic("syn");
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

#[proc_macro_attribute]
pub fn anyhow_result(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut our_func = syn::parse_macro_input!(item as syn::ItemFn);

    // let parent_crate = external_crate_parent();
    let quote_crate = quote_crate();
    let syn_crate = syn_crate();
    let proc_macro2_crate = proc_macro2_crate();
    let anyhow_crate = anyhow_crate()
        .to_string()
        .replace(|c: char| c.is_whitespace(), "");

    //Check if output of our function is a anyhow::Result<TokenStream>
    let func_output = &our_func.sig.output;
    match func_output {
        syn::ReturnType::Default => {
            panic!(
                "Function must return a {}::Result<TokenStream>",
                anyhow_crate
            )
        }
        syn::ReturnType::Type(_, ty) => {
            let ty_str = ty
                .to_token_stream()
                .to_string()
                .replace(|c: char| c.is_whitespace(), "");
            if ty_str != format!("{}::Result<TokenStream>", anyhow_crate)
                && ty_str != format!("{}::Result<proc_macro::TokenStream>", anyhow_crate)
            {
                panic!(
                    "Function must return a {}::Result<TokenStream>",
                    anyhow_crate
                );
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
                let mut result=#quote_crate::quote! {compile_error!};

                //Adds (formatted_error) to the end of the result
                result.extend( #proc_macro2_crate::TokenStream::from(#proc_macro2_crate::TokenTree::Group(#proc_macro2_crate::Group::new(
                    #proc_macro2_crate::Delimiter::Parenthesis,
                    #syn_crate::LitStr::new(&formatted_error, #proc_macro2_crate::Span::call_site()).into_token_stream(),
                ))));

                result.extend(#quote_crate::quote! {;});

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
                    let mut result = #quote_crate::quote! {compile_error!};

                    //Adds (formatted_error) to the end of the result
                    result.extend( #proc_macro2_crate::TokenStream::from(#proc_macro2_crate::TokenTree::Group(#proc_macro2_crate::Group::new(
                        #proc_macro2_crate::Delimiter::Parenthesis,
                        #syn_crate::LitStr::new(&formatted_error, #proc_macro2_crate::Span::call_site()).into_token_stream(),
                    ))));

                    result.extend(#quote_crate::quote! {;});

                    result.extend(#proc_macro2_crate::TokenStream::from(#second_input_arg));
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

    // Extract doc comments and other attributes to copy to the wrapper, excluding the proc_macro attribute
    let wrapper_attrs = our_func.attrs.iter().enumerate().filter_map(|(i, attr)| {
        // Skip the attribute we just removed (adjust index since we already removed one)
        if i == attr_index { None } else { Some(attr) }
    });

    let result = quote::quote! {
        #(#wrapper_attrs)*
        #macro_attr
        pub fn #func_name(#inputs) -> proc_macro::TokenStream {

            use #quote_crate::ToTokens;

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
