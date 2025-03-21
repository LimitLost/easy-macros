mod context_arg;

use all_syntax_cases::all_syntax_cases;
use context_arg::arg_handle;
use helpers_macro_safe::{ErrorData, expr_error_wrap, readable_token_stream};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Macro, punctuated::Punctuated};

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

pub fn context_no_func_input(
    expr: Box<syn::Expr>,
    question_span: proc_macro2::Span,
) -> Box<syn::Expr> {
    context_base(expr, question_span, Default::default())
}

struct FoundContextInfo {
    ///If None show errors on unsupported Expr's
    call_found: Option<TokenStream>,
    ///Contains errors that should be shown by expr_error_wrap with compile_error!()
    current_errors: Vec<String>,
    inputs_found: Vec<TokenStream>,
}

impl ErrorData for FoundContextInfo {
    fn no_errors(&self) -> bool {
        self.current_errors.is_empty()
    }

    fn error_data(&mut self) -> Vec<String> {
        self.current_errors.error_data()
    }
}
//before ?
all_syntax_cases! {
    setup => {
        generated_fn_prefix: "get_context",
        additional_input_type: &mut FoundContextInfo
    }
    default_cases => {
        #[after_system]
        fn expr_error_wrap(expr: &mut Expr, context_info: &mut FoundContextInfo);
    }
    special_cases => {
        fn context_call_handle(call: &mut syn::ExprCall, context_info: &mut FoundContextInfo);
        fn context_method_call_handle(method_call: &mut syn::ExprMethodCall, context_info: &mut FoundContextInfo);
        fn context_block_handle(block: &mut syn::ExprBlock, context_info: &mut FoundContextInfo);
        fn context_if_handle(if_: &mut syn::ExprIf, context_info: &mut FoundContextInfo);
        fn context_match_handle(match_: &mut syn::ExprMatch, context_info: &mut FoundContextInfo);
        fn context_while_handle(while_: &mut syn::ExprWhile, context_info: &mut FoundContextInfo);
        fn context_field_handle(field: &mut syn::ExprField, context_info: &mut FoundContextInfo);
        fn context_for_loop_handle(for_loop: &mut syn::ExprForLoop, context_info: &mut FoundContextInfo);
        fn context_loop_handle(loop_: &mut syn::ExprLoop, context_info: &mut FoundContextInfo);
        fn context_macro_handle(macro_: &mut syn::ExprMacro, context_info: &mut FoundContextInfo);
        fn context_path_handle(path: &mut syn::ExprPath, context_info: &mut FoundContextInfo);
    }
}

fn context_call_handle(call: &mut syn::ExprCall, context_info: &mut FoundContextInfo) {
    //Anyhow method, go deeper
    let func_str = call.func.to_token_stream().to_string();

    if func_str.ends_with(". with_context") || func_str.ends_with(". context") {
        get_context_expr_handle(&mut call.func, context_info);
        return;
    }

    for arg in call.args.iter_mut() {
        arg_handle(arg, context_info);
    }

    context_info.call_found = Some(call.func.to_token_stream());
}

fn context_method_call_handle(
    method_call: &mut syn::ExprMethodCall,
    context_info: &mut FoundContextInfo,
) {
    //Anyhow method, go deeper
    if let "with_context" | "context" = method_call.method.to_string().as_str() {
        get_context_expr_handle(&mut method_call.receiver, context_info);
        return;
    }

    for arg in method_call.args.iter_mut() {
        arg_handle(arg, context_info);
    }

    let before_dot = &method_call.receiver;
    let after_dot = &method_call.method;

    context_info.call_found = Some(quote! { #before_dot.#after_dot });
}

//Handle ExprBlock (raise error (unsupported syntax))
fn context_block_handle(_block: &mut syn::ExprBlock, context_info: &mut FoundContextInfo) {
    context_info
        .current_errors
        .push("Always Context Macro: ExprBlock right before '?' is not supported".to_string());
}
// Handle ExprIf (raise error (unsupported syntax))
fn context_if_handle(_if: &mut syn::ExprIf, context_info: &mut FoundContextInfo) {
    context_info
        .current_errors
        .push("Always Context Macro: ExprIf right before '?' is not supported".to_string());
}
// Handle ExprMatch (raise error (unsupported syntax))
fn context_match_handle(_match: &mut syn::ExprMatch, context_info: &mut FoundContextInfo) {
    context_info
        .current_errors
        .push("Always Context Macro: ExprMatch right before '?' is not supported".to_string());
}
// Handle ExprWhile (raise error (unsupported syntax))
fn context_while_handle(_while: &mut syn::ExprWhile, context_info: &mut FoundContextInfo) {
    context_info
        .current_errors
        .push("Always Context Macro: ExprWhile right before '?' is not supported".to_string());
}
// Handle ExprField (raise error (unsupported syntax))
fn context_field_handle(_field: &mut syn::ExprField, context_info: &mut FoundContextInfo) {
    context_info
        .current_errors
        .push("Always Context Macro: ExprField right before '?' is not supported".to_string());
}
// Handle ExprForLoop (raise error (unsupported syntax))
fn context_for_loop_handle(_for_loop: &mut syn::ExprForLoop, context_info: &mut FoundContextInfo) {
    context_info
        .current_errors
        .push("Always Context Macro: ExprForLoop right before '?' is not supported".to_string());
}
// Handle ExprLoop (raise error (unsupported syntax))
fn context_loop_handle(_loop: &mut syn::ExprLoop, context_info: &mut FoundContextInfo) {
    context_info
        .current_errors
        .push("Always Context Macro: ExprLoop right before '?' is not supported".to_string());
}
// Handle ExprMacro (raise error (unsupported syntax))
fn context_macro_handle(_macro: &mut syn::ExprMacro, context_info: &mut FoundContextInfo) {
    context_info
        .current_errors
        .push("Always Context Macro: ExprMacro right before '?' is not supported".to_string());
}
// Handle ExprPath (raise error (unsupported syntax))
fn context_path_handle(_path: &mut syn::ExprPath, context_info: &mut FoundContextInfo) {
    context_info
        .current_errors
        .push("Always Context Macro: ExprPath right before '?' is not supported".to_string());
}

pub fn context(mut expr: Box<syn::Expr>, question_span: proc_macro2::Span) -> Box<syn::Expr> {
    let mut found_context_info = FoundContextInfo {
        call_found: None,
        current_errors: vec![],
        inputs_found: vec![],
    };

    get_context_expr_handle(&mut expr, &mut found_context_info);
    expr_error_wrap(&mut expr, &mut found_context_info);

    let mut macro_input = TokenStream::new();

    if let Some(call_found) = found_context_info.call_found {
        let inputs_found = found_context_info.inputs_found;
        // This adds default into_token_stream() spaces, which quote!{} macro doesn't do
        let quote_parsed: syn::Expr = syn::parse_quote! {#call_found(#(#inputs_found),*)};
        let mut call_str = readable_token_stream(&quote_parsed.into_token_stream().to_string());
        if !inputs_found.is_empty() {
            call_str.push_str("\r\n\r\nArguments:\r\n");
            //Add arguments to call_str in format: "argument: {}"
            for input in &inputs_found {
                call_str.push_str(&format!(
                    "{}: {{}}\r\n\r\n",
                    readable_token_stream(&input.to_token_stream().to_string())
                ));
            }
        }

        let format_str_arg = syn::LitStr::new(&call_str, question_span);
        macro_input.extend(quote! { #format_str_arg });

        if !inputs_found.is_empty() {
            macro_input.extend(quote! { , });

            macro_input.extend(quote! { #(#inputs_found),* });
        }
    }

    context_base(
        expr,
        question_span,
        quote::quote_spanned! {question_span=>#macro_input},
    )
}
