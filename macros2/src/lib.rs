use proc_macro::{TokenStream};
use quote::{ToTokens, quote};
use syn::{spanned::Spanned, AngleBracketedGenericArguments, Token};

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

fn always_context_generic_args(turbofish:&mut AngleBracketedGenericArguments,no_context: &Option<NoContext>){
    for arg in turbofish.args.iter_mut(){
        match arg{
            syn::GenericArgument::Lifetime(_) => {},
            syn::GenericArgument::Type(_) => {},
            syn::GenericArgument::Const(expr) => {
                always_context_expr(expr, &no_context);
            },
            syn::GenericArgument::AssocType(assoc_type) => {
                if let Some(t)=&mut assoc_type.generics{
                    always_context_generic_args(t, no_context);
                }
            },
            syn::GenericArgument::AssocConst(assoc_const) => {
                if let Some(t)=&mut assoc_const.generics{
                    always_context_generic_args(t, no_context);
                }
                always_context_expr(&mut assoc_const.value, no_context);
            },
            syn::GenericArgument::Constraint(constraint) => {
                if let Some(t)=&mut constraint.generics{
                    always_context_generic_args(t, no_context);
                }
            },
            a => todo!("Not implemented yet in always_context: {}", a.to_token_stream()),
        }
    }
}

fn always_context_found(expr:&mut syn::ExprTry, no_context: &Option<NoContext>) {
    match no_context{
        Some(NoContext::All) => {
            //No context, don't do anything
        },
        Some(NoContext::NoFuncInput) => {
            //Don't put function names and inputs in `context!(...)``
            expr.expr=Box::new(syn::Expr::MethodCall( syn::ExprMethodCall{
                attrs: vec![],
                receiver: expr.expr,
                dot_token: Default::default(),
                method: quote::format_ident!("with_context"),
                turbofish: None,
                paren_token: Default::default(),
                args: todo!(),//TODO AAAAAAAAAAAAAAAAAAAAAAAAA
            }));
            
        },
        Some(NoContext::EnableBack) | None => todo!(),
    }
}

//TODO Macro idea, handle all attributes and expressions, statements, items, allow for exceptions to the rule

fn always_context_expr(expr: &mut syn::Expr, no_context: &Option<NoContext>) {
    match expr {
        syn::Expr::Array(expr_array) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_array.attrs) {
                no_context = Some(no_c);
            }
            for el in expr_array.elems.iter_mut() {
                always_context_expr(el, &no_context);
            }
        }
        syn::Expr::Assign(expr_assign) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_assign.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_assign.left, &no_context);
            always_context_expr(&mut expr_assign.right, &no_context);
        },
        syn::Expr::Async(expr_async) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_async.attrs) {
                no_context = Some(no_c);
            }
            always_context_block(&mut expr_async.block, &no_context);
        },
        syn::Expr::Await(expr_await) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_await.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_await.base, &no_context);
        },
        syn::Expr::Binary(expr_binary) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_binary.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_binary.left, &no_context);
            always_context_expr(&mut expr_binary.right, &no_context);
        },
        syn::Expr::Block(expr_block) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_block.attrs) {
                no_context = Some(no_c);
            }
            always_context_block(&mut expr_block.block, &no_context);
        },
        syn::Expr::Break(expr_break) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_break.attrs) {
                no_context = Some(no_c);
            }
            if let Some(expr) = &mut expr_break.expr {
                always_context_expr(expr, &no_context);
            }
        },
        syn::Expr::Call(expr_call) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_call.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_call.func, &no_context);
            for arg in expr_call.args.iter_mut() {
                always_context_expr(arg, &no_context);
            }
        },
        syn::Expr::Cast(expr_cast) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_cast.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_cast.expr, &no_context);
        },
        syn::Expr::Closure(expr_closure) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_closure.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_closure.body, &no_context);
        },
        syn::Expr::Const(expr_const) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_const.attrs) {
                no_context = Some(no_c);
            }
            always_context_block(&mut expr_const.block, &no_context);
        },
        syn::Expr::Continue(_) => {
            //No Expr inside, Move Along
        },
        syn::Expr::Field(expr_field) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_field.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_field.base, &no_context);
        },
        syn::Expr::ForLoop(expr_for_loop) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_for_loop.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_for_loop.expr, &no_context);
            always_context_block(&mut expr_for_loop.body, &no_context);
        },
        syn::Expr::Group(expr_group) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_group.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_group.expr, &no_context);
        },
        syn::Expr::If(expr_if) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_if.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_if.cond, &no_context);
            always_context_block(&mut expr_if.then_branch, &no_context);
            if let Some((_, block)) = &mut expr_if.else_branch {
                always_context_expr(block, &no_context);
            }
        },
        syn::Expr::Index(expr_index) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_index.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_index.expr, &no_context);
            always_context_expr(&mut expr_index.index, &no_context);
        },
        syn::Expr::Infer(_) => {
            //No Expr inside, Move Along
        },
        syn::Expr::Let(expr_let) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_let.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_let.expr, &no_context);
        },
        syn::Expr::Lit(_) => {
            //No Expr inside, Move Along
        },
        syn::Expr::Loop(expr_loop) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_loop.attrs) {
                no_context = Some(no_c);
            }
            always_context_block(&mut expr_loop.body, &no_context);
        },
        syn::Expr::Macro(expr_macro) => {
            //Enable only if we have #[enable_context], support only for stmts (statements)
            let mut no_context = NoContext::All;
            if let Some(no_c) = always_context_attr_check(&mut expr_macro.attrs) {
                no_context = no_c;
            }

            if let NoContext::All = no_context{
                return;
            }

            let mut parsed=match syn::parse2::<syn::Stmt>(expr_macro.mac.tokens.clone()){
                Ok(parsed)=>parsed,
                Err(e) => {
                    panic!("Expected Statement, error while parsing: {} | tried to parse: {}",e,expr_macro.to_token_stream());
                }
            };

            always_context_stmt(&mut parsed, &Some(no_context));

            expr_macro.mac.tokens=parsed.into_token_stream();
        },
        syn::Expr::Match(expr_match) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_match.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_match.expr, &no_context);
            for arm in expr_match.arms.iter_mut() {
                let mut no_context = no_context;
                if let Some(no_c) = always_context_attr_check(&mut arm.attrs) {
                    no_context = Some(no_c);
                }
                if let Some((_,ex))= &mut arm.guard {
                    always_context_expr( ex, &no_context);
                }
                
                always_context_expr(&mut arm.body, &no_context);
            }
        },
        syn::Expr::MethodCall(expr_method_call) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_method_call.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_method_call.receiver, &no_context);
            for arg in expr_method_call.args.iter_mut() {
                always_context_expr(arg, &no_context);
            }
            if let Some(turbofish) = &mut expr_method_call.turbofish {
                always_context_generic_args(turbofish, &no_context);
            }

        },
        syn::Expr::Paren(expr_paren) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_paren.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_paren.expr, &no_context);
        },
        syn::Expr::Path(_) => {
            //No Expr inside, Move Along
        },
        syn::Expr::Range(expr_range) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_range.attrs) {
                no_context = Some(no_c);
            }
            if let Some(from) = &mut expr_range.start {
                always_context_expr(from, &no_context);
            }
            if let Some(to) = &mut expr_range.end {
                always_context_expr(to, &no_context);
            }
        },
        syn::Expr::RawAddr(expr_raw_addr) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_raw_addr.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_raw_addr.expr, &no_context);
        },
        syn::Expr::Reference(expr_reference) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_reference.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_reference.expr, &no_context);
        },
        syn::Expr::Repeat(expr_repeat) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_repeat.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_repeat.len, &no_context);
            always_context_expr(&mut expr_repeat.expr, &no_context);
        },
        syn::Expr::Return(expr_return) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_return.attrs) {
                no_context = Some(no_c);
            }
            if let Some(expr) = &mut expr_return.expr {
                always_context_expr(expr, &no_context);
            }
        },
        syn::Expr::Struct(expr_struct) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_return.attrs) {
                no_context = Some(no_c);
            }

            if let Some(rest)=&mut expr_struct.rest {
                always_context_expr(rest, &no_context);
            }

            for field in expr_struct.fields.iter_mut() {
                let mut no_context = no_context;
                if let Some(no_c) = always_context_attr_check(&mut field.attrs) {
                    no_context = Some(no_c);
                }
                always_context_expr(&mut field.expr, &no_context);
            }
        },
        syn::Expr::Try(expr_try) => {
            let mut no_context = *no_context;
            if let Some(no_c) = always_context_attr_check(&mut expr_try.attrs) {
                no_context = Some(no_c);
            }
            always_context_expr(&mut expr_try.expr, &no_context);
        },
        syn::Expr::TryBlock(expr_try_block) => todo!(),
        syn::Expr::Tuple(expr_tuple) => todo!(),
        syn::Expr::Unary(expr_unary) => todo!(),
        syn::Expr::Unsafe(expr_unsafe) => todo!(),
        syn::Expr::Verbatim(token_stream) => todo!(),
        syn::Expr::While(expr_while) => todo!(),
        syn::Expr::Yield(expr_yield) => todo!(),
        _ => todo!(),
    }
}

fn always_context_stmt(stmt: &mut syn::Stmt, no_context: &Option<NoContext>) {
    match stmt {
        syn::Stmt::Local(local) => todo!()
        syn::Stmt::Item(item) => todo!(),
        syn::Stmt::Expr(expr, _) => {
            always_context_expr(expr, no_context);
        }
        syn::Stmt::Macro(stmt_macro) => todo!(),
    }
}

fn always_context_block(block:&mut syn::Block,no_context: &Option<NoContext>){
    for stmt in block.stmts.iter_mut() {
        always_context_stmt(stmt, no_context);
    }
}

#[proc_macro_attribute]
///Adds .with_context(context!()) before all '?' without them
pub fn always_context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as syn::ItemFn);
    //Adds .with_context(context!()) before all '?' without them
    //Maybe add also function inputs with names into context?

    always_context_block(&mut parsed.block, &None);

    //TODO #[no_context] attribute, when we don't want context from this but our own?
    //TODO #[no_context_inputs] attribute, when we don't want function inputs in context
    todo!()
}

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

#[proc_macro_attribute]
///Creates a wrapper for passed in function, passed in function is placed inside of wrapper
pub fn macro_result(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut our_func = syn::parse_macro_input!(item as syn::ItemFn);

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
            if ty_str != "anyhow::Result<TokenStream>" {
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
                err_result = Some(
                    quote::quote! { quote::quote! {compile_error!("{}\r\n\r\nDebug Info: {:?}", ___macro_err, ___macro_err);} },
                );
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
                   let mut result = quote::quote! {compile_error!("{}\r\n\r\nDebug Info: {:?}", ___macro_err, ___macro_err);}

                   result.extend(proc_macro2::TokenStream::from(#second_input_arg));
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
