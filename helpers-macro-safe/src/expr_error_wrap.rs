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
/// ```rust
/// use easy_macros_helpers_macro_safe::ErrorData;
///
/// let mut errors = Vec::<String>::new();
/// errors.push("Invalid syntax".to_string());
/// errors.push("Missing required field".to_string());
///
/// assert!(!errors.no_errors());
/// let error_messages = errors.error_data();
/// assert!(errors.no_errors()); // Errors are consumed
/// ```
///
/// ## Custom Implementation
///
/// ```rust
/// use easy_macros_helpers_macro_safe::ErrorData;
///
/// #[derive(Default)]
/// struct ValidationErrors {
///     errors: Vec<String>,
///     other_data: String,
/// }
///
/// impl ValidationErrors {
///     fn add_error(&mut self, msg: &str) {
///         self.errors.push(msg.to_string());
///     }
/// }
///
/// impl ErrorData for ValidationErrors {
///     fn no_errors(&self) -> bool {
///         self.errors.no_errors()
///     }
///
///     fn error_data(&mut self) -> Vec<String> {
///         self.errors.error_data()
///     }
/// }
///
/// let mut validator = ValidationErrors::default();
/// validator.add_error("Field 'name' is required");
/// validator.add_error("Field 'age' must be a positive number");
///
/// assert!(!validator.no_errors());
/// let messages = validator.error_data();
/// assert_eq!(messages.len(), 2);
/// assert!(validator.no_errors()); // All errors consumed
/// ```
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
/// ```rust
/// use easy_macros_helpers_macro_safe::{expr_error_wrap, ErrorData};
/// use syn::parse_quote;
///
/// let mut expr = parse_quote!(42);
/// let mut errors = vec![
///     "This is a warning".to_string(),
///     "Another issue found".to_string(),
/// ];
///
/// expr_error_wrap(&mut expr, &mut errors);
///
/// // The expression is now wrapped with compile errors:
/// // {
/// //     compile_error!("This is a warning");
/// //     compile_error!("Another issue found");
/// //     42
/// // }
/// ```
///
/// ## Using Custom ErrorData Implementation
///
/// ```rust
/// use easy_macros_helpers_macro_safe::{expr_error_wrap, ErrorData};
/// use syn::parse_quote;
///
/// #[derive(Default)]
/// struct MacroValidator {
///     errors: Vec<String>,
///     context: String,
/// }
///
/// impl MacroValidator {
///     fn new(context: &str) -> Self {
///         Self {
///             errors: Vec::new(),
///             context: context.to_string(),
///         }
///     }
///     
///     fn validate_field(&mut self, field_name: &str, is_valid: bool) {
///         if !is_valid {
///             self.errors.push(format!("Invalid field '{}' in {}", field_name, self.context));
///         }
///     }
/// }
///
/// impl ErrorData for MacroValidator {
///     fn no_errors(&self) -> bool {
///         self.errors.no_errors()
///     }
///
///     fn error_data(&mut self) -> Vec<String> {
///         self.errors.error_data()
///     }
/// }
///
/// // Usage in a procedural macro context
/// let mut validator = MacroValidator::new("MyStruct");
/// validator.validate_field("name", false);
/// validator.validate_field("id", false);
///
/// let mut result_expr = parse_quote!(MyStruct::default());
/// expr_error_wrap(&mut result_expr, &mut validator);
///
/// // The expression now includes compile-time validation errors
/// ```
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
