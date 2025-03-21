use proc_macro::TokenStream;
use syn::Token;

///Same input as format! macro
struct ContextInternalInput {
    str: syn::LitStr,
    _comma: Option<Token![,]>,
    args: syn::punctuated::Punctuated<syn::Expr, Token![,]>,
}

enum ContextInternalMaybeInput {
    Yes(ContextInternalInput),
    No,
}

impl syn::parse::Parse for ContextInternalInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        //Handle no input
        if input.is_empty() {
            return Ok(ContextInternalInput {
                str: syn::LitStr::new("", proc_macro2::Span::call_site()),
                _comma: None,
                args: syn::punctuated::Punctuated::new(),
            });
        }
        let str = input.parse()?;
        if !input.is_empty() {
            let _comma = input.parse()?;
            let args = input.parse_terminated(syn::Expr::parse, Token![,])?;
            Ok(ContextInternalInput { str, _comma, args })
        } else {
            Ok(ContextInternalInput {
                str,
                _comma: None,
                args: syn::punctuated::Punctuated::new(),
            })
        }
    }
}

impl syn::parse::Parse for ContextInternalMaybeInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(ContextInternalMaybeInput::No);
        }
        Ok(ContextInternalMaybeInput::Yes(input.parse()?))
    }
}
#[proc_macro]
///Use context! macro from helpers crate instead
pub fn context_internal(item: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(item as ContextInternalMaybeInput);

    let (mut passed_in_str, mut passed_in_args) = match parsed {
        ContextInternalMaybeInput::Yes(context_internal_input) => (
            context_internal_input.str.value(),
            context_internal_input.args,
        ),
        ContextInternalMaybeInput::No => (String::new(), syn::punctuated::Punctuated::new()),
    };
    if passed_in_str.is_empty() {
        passed_in_str = "{}:{}".to_owned();
    } else {
        passed_in_str = format!("{{}}:{{}}\r\n{}", passed_in_str);
    }
    passed_in_args.insert(
        0,
        syn::parse_quote! {
            file!()
        },
    );
    passed_in_args.insert(
        1,
        syn::parse_quote! {
            line!()
        },
    );

    let result = quote::quote! {
        format!(#passed_in_str, #passed_in_args)
    };

    // panic!("{}", result.to_string());

    result.into()
}

#[test]
fn format_compiler_test() {
    let test_str = "Str";
    let _ = format!("file: {}:{} | {test_str} | ", file!(), line!());
    let _ = format!("{} | file: {}:{}", test_str, file!(), line!());
}
