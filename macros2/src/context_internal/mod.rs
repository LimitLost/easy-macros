use proc_macro::TokenStream;
use syn::Token;

///Same input as format! macro
struct ContextInternalInput {
    str: syn::LitStr,
    _comma: Token![,],
    args: syn::punctuated::Punctuated<syn::Expr, Token![,]>,
}

enum ContextInternalMaybeInput {
    Yes(ContextInternalInput),
    No,
}

impl syn::parse::Parse for ContextInternalInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let str = input.parse()?;
        let _comma = input.parse()?;
        let args = input.parse_terminated(syn::Expr::parse, Token![,])?;
        Ok(ContextInternalInput { str, _comma, args })
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
        passed_in_str = "file: {}:{}".to_owned();
    } else {
        if passed_in_str.contains(|c: char| c == '\r' || c == '\n') {
            passed_in_str = format!("{} \r\n\r\n file: {{}}:{{}}", passed_in_str);
        } else {
            passed_in_str = format!("{} | file: {{}}:{{}}", passed_in_str);
        }
    }
    passed_in_args.push(syn::parse_quote! {
        file!()
    });
    passed_in_args.push(syn::parse_quote! {
        line!()
    });

    let result = quote::quote! {
        format!(#passed_in_str, #passed_in_args)
    };

    // panic!("{}", result.to_string());

    result.into()
}

#[test]
fn format_compiler_test() {
    let test_str = "Str";
    let _ = format!("{test_str} | file: {}:{}", file!(), line!());
    let _ = format!("{} | file: {}:{}", test_str, file!(), line!());
}
