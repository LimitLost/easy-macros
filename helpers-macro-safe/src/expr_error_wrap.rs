use syn::{Block, Expr, ExprBlock, spanned::Spanned};

pub trait ErrorData {
    fn no_errors(&self) -> bool;

    /// Calling this should remove all the error data from vector and return it
    ///
    /// Vec<String> also implements this trait, so you can use it directly
    fn error_data(&mut self) -> Vec<String>;
}

impl ErrorData for Vec<String> {
    fn no_errors(&self) -> bool {
        self.is_empty()
    }

    fn error_data(&mut self) -> Vec<String> {
        let mut data = Vec::new();
        std::mem::swap(self, &mut data);
        data
    }
}

///Wraps expression with block with added compile_error!() if needed
pub fn expr_error_wrap(expr: &mut Expr, error_info: &mut impl ErrorData) {
    if !error_info.no_errors() {
        let errors = error_info.error_data();

        let span = expr.span();

        let mut error_calls = errors
            .iter()
            .map(|error| {
                let error: syn::Stmt = syn::parse_quote_spanned! {span=>
                    compile_error!(#error);
                };
                error
            })
            .collect::<Vec<_>>();

        replace_with::replace_with_or_abort(expr, |ex| {
            error_calls.push(syn::Stmt::Expr(ex, None));

            Expr::Block(ExprBlock {
                attrs: vec![],
                label: None,
                block: Block {
                    brace_token: Default::default(),
                    stmts: error_calls,
                },
            })
        });
    }
}
