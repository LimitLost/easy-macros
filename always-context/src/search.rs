use all_syntax_cases::all_syntax_cases;
use quote::ToTokens;
use syn::{ItemImpl, ItemTrait, TraitItem, Type, spanned::Spanned};

use crate::context_gen::{context, context_no_func_input};

#[derive(Debug, Clone, Copy)]
pub enum NoContext {
    ///Don't put .with_context(context!()) at all
    /// #[no_context] attribute
    All,
    ///Don't put function names and inputs in `context!(...)``
    ///#[no_context_inputs] attribute
    NoFuncInput,
    ///#[enable_context] attribute
    EnableBack,
}

fn always_context_attr_check(attrs: &mut Vec<syn::Attribute>) -> Option<NoContext> {
    for (index, attr) in attrs.iter().enumerate() {
        let attr_str = attr.to_token_stream().to_string();
        if attr_str == "#[no_context]" {
            attrs.remove(index);
            return Some(NoContext::All);
        } else if attr_str == "#[no_context_inputs]" {
            attrs.remove(index);
            return Some(NoContext::NoFuncInput);
        } else if attr_str == "#[enable_context]" {
            attrs.remove(index);
            return Some(NoContext::EnableBack);
        }
    }
    None
}

//always_context syntax cases
all_syntax_cases! {
    setup => {
        generated_fn_prefix: "always_context",
        additional_input_type: Option<NoContext>
    }
    default_cases => {
        fn handle_attributes(attrs: &mut Vec<syn::Attribute>, no_context: &mut Option<NoContext>);
    }
    special_cases => {
        fn always_context_try(expr_try: &mut syn::ExprTry, no_context: Option<NoContext>);
        fn always_context_macro(macro_: &mut syn::Macro, attrs: &mut Vec<syn::Attribute>);
        fn always_context_item_trait(item_trait: &mut ItemTrait, no_context: Option<NoContext>);
        fn always_context_item_impl(item_impl: &mut ItemImpl, no_context: Option<NoContext>);
    }
}

fn handle_attributes(attrs: &mut Vec<syn::Attribute>, no_context: &mut Option<NoContext>) {
    if let Some(no_c) = always_context_attr_check(attrs) {
        *no_context = Some(no_c);
    }
}

fn always_context_macro(macro_: &mut syn::Macro, attrs: &mut Vec<syn::Attribute>) {
    //Enable only if we have #[enable_context], support only for stmts (statements)
    let mut no_context = NoContext::All;
    if let Some(no_c) = always_context_attr_check(attrs) {
        no_context = no_c;
    }

    if let NoContext::All = no_context {
        return;
    }

    let mut parsed = match syn::parse2::<syn::Stmt>(macro_.tokens.clone()) {
        Ok(parsed) => parsed,
        Err(e) => {
            panic!(
                "Expected Statement, error while parsing: {} | tried to parse: {}",
                e,
                macro_.to_token_stream()
            );
        }
    };

    always_context_stmt_handle(&mut parsed, Some(no_context));

    macro_.tokens = parsed.into_token_stream();
}

fn always_context_try(expr: &mut syn::ExprTry, mut no_context: Option<NoContext>) {
    handle_attributes(&mut expr.attrs, &mut no_context);

    match no_context {
        Some(NoContext::All) => {
            //No context, don't do anything
        }
        Some(NoContext::NoFuncInput) => {
            //Don't put function names and inputs in `context!(...)``

            replace_with::replace_with_or_abort(&mut expr.expr, |ex| {
                context_no_func_input(ex, expr.question_token.span())
            });
        }
        Some(NoContext::EnableBack) | None => {
            //Put all info available into context

            replace_with::replace_with_or_abort(&mut expr.expr, |ex| {
                context(ex, expr.question_token.span())
            });
        }
    }
}
///Returns `true` if the type is `anyhow::Result`
fn anyhow_result_check(ty: &Type) -> bool {
    if let Type::Path(ty) = ty {
        let mut segments = ty.path.segments.iter();
        if let Some(segment) = segments.next() {
            if segment.ident != "anyhow" {
                return false;
            }
        } else {
            return false;
        }
        if let Some(segment) = segments.next() {
            if segment.ident == "Result" {
                return true;
            }
        }
    }
    false
}

fn always_context_item_trait(item_trait: &mut ItemTrait, mut no_context: Option<NoContext>) {
    let ItemTrait {
        attrs,
        vis: _,
        unsafety: _,
        auto_token: _,
        restriction: _,
        trait_token: _,
        ident: _,
        generics: _,
        colon_token: _,
        supertraits: _,
        brace_token: _,
        items,
    } = item_trait;

    handle_attributes(attrs, &mut no_context);

    for item in items.iter_mut() {
        if let TraitItem::Fn(f) = item {
            if let Some(block) = &mut f.default {
                match &mut f.sig.output {
                    syn::ReturnType::Default => {
                        //No return type, don't add ? anywhere
                    }
                    syn::ReturnType::Type(_, ty) => {
                        //Check if our type is anyhow::Result
                        if !anyhow_result_check(ty) {
                            continue;
                        }
                        //Attr check
                        let mut no_context = no_context;
                        handle_attributes(&mut f.attrs, &mut no_context);

                        //Add context to block
                        always_context_block_handle(block, no_context);
                    }
                }
            }
        }
    }
}

fn always_context_item_impl(item_impl: &mut ItemImpl, mut no_context: Option<NoContext>) {
    let ItemImpl {
        attrs,
        defaultness: _,
        unsafety: _,
        impl_token: _,
        generics: _,
        trait_: _,
        self_ty: _,
        brace_token: _,
        items,
    } = item_impl;

    handle_attributes(attrs, &mut no_context);

    for item in items.iter_mut() {
        if let syn::ImplItem::Fn(m) = item {
            match &mut m.sig.output {
                syn::ReturnType::Default => {
                    //No return type, don't add ? anywhere
                }
                syn::ReturnType::Type(_, ty) => {
                    //Check if our type is anyhow::Result
                    if !anyhow_result_check(ty) {
                        continue;
                    }
                    //Attr check
                    let mut no_context = no_context;
                    handle_attributes(&mut m.attrs, &mut no_context);

                    //Add context to block
                    always_context_block_handle(&mut m.block, no_context);
                }
            }
        }
    }
}

pub fn item_handle(item: &mut syn::Item, no_context: Option<NoContext>) {
    always_context_item_handle(item, no_context);
}
