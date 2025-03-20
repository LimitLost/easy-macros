use all_syntax_cases::all_syntax_cases;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Expr, Macro, punctuated::Punctuated, spanned::Spanned};

#[derive(Debug, Clone, Copy)]
enum NoContext {
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

fn context_base(
    expr: Box<syn::Expr>,
    question_span: proc_macro2::Span,
    context_macro_input: proc_macro2::TokenStream,
) -> Box<syn::Expr> {
    let mut punc = Punctuated::new();
    punc.push(Expr::Macro(syn::ExprMacro {
        attrs: vec![],
        mac: Macro {
            path: syn::parse_quote_spanned! {question_span=>
                context
            },
            bang_token: Default::default(),
            delimiter: syn::MacroDelimiter::Paren(Default::default()),
            tokens: context_macro_input,
        },
    }));

    Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
        attrs: vec![],
        receiver: expr,
        dot_token: Default::default(),
        method: quote::format_ident!("with_context", span = question_span),
        turbofish: None,
        paren_token: Default::default(),
        args: punc,
    }))
}

all_syntax_cases! {
    setup => {
        generated_fn_prefix: "always_context",
        additional_input_type: Option<NoContext>
    }
    default_cases => {
        fn handle_attributes(attrs: &mut Vec<syn::Attribute>, no_context: &mut Option<NoContext>);
    }
    special_cases => {
        fn always_context_try(expr_try: &mut syn::ExprTry, no_context: Option<NoContext>) ;
        fn always_context_macro(macro_: &mut syn::Macro, attrs: &mut Vec<syn::Attribute>) ;
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

fn context_no_func_input(expr: Box<syn::Expr>, question_span: proc_macro2::Span) -> Box<syn::Expr> {
    context_base(expr, question_span, Default::default())
}

fn context(expr: Box<syn::Expr>, question_span: proc_macro2::Span) -> Box<syn::Expr> {
    let mut format_str = String::new();

    //TODO Find all variables used in our expr

    context_base(expr, question_span, quote::quote_spanned! {question_span=>})
}

pub fn always_context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as syn::Item);
    //Adds .with_context(context!()) before all '?' without them
    //Maybe add also function inputs with names into context?

    always_context_item_handle(&mut parsed, None);

    //TODO #[no_context] attribute, when we don't want context from this but our own?
    //TODO #[no_context_inputs] attribute, when we don't want function inputs in context

    parsed.into_token_stream().into()
}
