use anyhow::bail;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_quote, Block, Ident, Token};

#[derive(Default)]
struct AddCodeArgs {
    before: Option<Vec<syn::Stmt>>,
    after: Option<Vec<syn::Stmt>>,
}

impl Parse for AddCodeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = AddCodeArgs::default();

        while !input.is_empty() {
            let name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            let block: Block = input.parse()?;
            let stmts = block.stmts;

            match name.to_string().as_str() {
                "before" => {
                    if args.before.is_some() {
                        return Err(syn::Error::new(name.span(), "duplicate `before`"));
                    }
                    args.before = Some(stmts);
                }
                "after" => {
                    if args.after.is_some() {
                        return Err(syn::Error::new(name.span(), "duplicate `after`"));
                    }
                    args.after = Some(stmts);
                }
                _ => {
                    return Err(syn::Error::new(name.span(), "expected `before` or `after`"));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if args.before.is_none() && args.after.is_none() {
            return Err(syn::Error::new(input.span(), "missing `before` or `after`"));
        }

        Ok(args)
    }
}

fn inject_into_block(block: &mut Block, args: &AddCodeArgs) {
    if let Some(before) = &args.before {
        let mut before_stmts = before.clone();
        block.stmts.splice(0..0, before_stmts.drain(..));
    }

    if let Some(after) = &args.after {
        let tail_expr = match block.stmts.last() {
            Some(syn::Stmt::Expr(expr, None)) => Some(expr.clone()),
            _ => None,
        };

        if let Some(expr) = tail_expr {
            block.stmts.pop();
            block
                .stmts
                .push(parse_quote! { let __add_code_result = { #expr }; });
            block.stmts.extend(after.clone());
            let tail_expr: syn::Expr = parse_quote! { __add_code_result };
            block.stmts.push(syn::Stmt::Expr(tail_expr, None));
        } else {
            block.stmts.extend(after.clone());
        }
    }
}

#[proc_macro_attribute]
#[anyhow_result::anyhow_result]
/// Injects code at the beginning and/or end of a function-like item.
///
/// This is handy for keeping [docify](https://crates.io/crates/docify)-generated examples clean while still
/// inserting setup/teardown or assertions for tests.
///
/// # Syntax
/// ```rust,ignore
/// #[add_code(before = { /* statements */ })]
/// #[add_code(after = { /* statements */ })]
/// #[add_code(before = { ... }, after = { ... })]
/// fn demo() { /* ... */ }
/// ```
///
/// The statements inside the braces are inserted without the outer `{}`.
pub fn add_code(attr: TokenStream, item: TokenStream) -> anyhow::Result<TokenStream> {
    let args = helpers::parse_macro_input!(attr as AddCodeArgs);

    let item_ts: proc_macro2::TokenStream = item.clone().into();

    if let Ok(mut item_fn) = syn::parse2::<syn::ItemFn>(item_ts.clone()) {
        inject_into_block(&mut item_fn.block, &args);
        return Ok(quote! { #item_fn }.into());
    }

    if let Ok(mut impl_fn) = syn::parse2::<syn::ImplItemFn>(item_ts) {
        inject_into_block(&mut impl_fn.block, &args);
        return Ok(quote! { #impl_fn }.into());
    }

    bail!("#[add_code] can only be used on functions or impl methods");
}
