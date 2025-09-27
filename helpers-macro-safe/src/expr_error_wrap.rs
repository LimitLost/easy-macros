use syn::{Block, Expr, ExprBlock, spanned::Spanned};

/// Collect and provide error information for [`expr_error_wrap`].
///
/// Types implementing this trait can accumulate error messages
/// and then show errors at the problematic code block by using [`expr_error_wrap`].
///
/// # Examples
///
/// ## Basic Usage with `Vec<String>`
///
#[doc = docify::embed!("src/examples.rs", error_data_basic_usage)]
///
/// ## Custom Implementation
///
#[doc = docify::embed!("src/examples.rs", error_data_custom_implementation)]
pub trait ErrorData {
    /// Returns `true` if there are no errors, `false` otherwise.
    ///
    /// This method is used to check whether error wrapping is needed.
    fn no_errors(&self) -> bool;

    /// Removes and returns all error data from the collection.
    ///
    /// After calling this method, the error collection should be empty.
    /// The returned vector contains all accumulated error messages.
    ///
    /// # Note
    ///
    /// `Vec<String>` implements this trait, so you can use it directly
    /// for simple error collection.
    ///
    /// # Returns
    ///
    /// A vector of error messages that were accumulated
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

/// Wraps an expression in a block that includes compile-time error messages.
///
/// This function is useful when you want to show that expression is problematic
/// It transforms a single expression into a block containing `compile_error!`
/// calls followed by the original expression.
///
/// # Arguments
///
/// * `expr` - The expression to wrap (will be modified in place)
/// * `error_info` - A struct or collection implementing [`ErrorData`]
///
/// # Behavior
///
/// If there are no errors in `error_info`, the expression is left unchanged.
/// If there are errors, the expression is wrapped in a block containing:
/// 1. `compile_error!` statements for each error message
/// 2. The original expression as the final statement
///
/// # Examples
///
/// ## Basic Usage
///
#[doc = docify::embed!("src/examples.rs", expr_error_wrap_basic_usage)]
///
/// ## Using Custom ErrorData Implementation
///
#[doc = docify::embed!("src/examples.rs", expr_error_wrap_custom_validator)]
///
/// # Use Cases
///
/// - Warning about deprecated or problematic usage patterns
/// - Validating macro input and reporting multiple issues at once
/// - Creating compile-time assertions with custom messages
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
