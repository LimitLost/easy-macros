use proc_macro::TokenStream;

mod all_syntax_cases;
mod helpers;

#[proc_macro]
/// Generates exhaustive handler functions for traversing syn AST syntax trees.
///
/// This macro creates a complete set of recursive handler functions that traverse through
/// all variants of syn's AST types (Item, Expr, Stmt, Pat, Type, etc.). It generates match
/// arms for every syn type variant and automatically routes to user-defined handlers.
///
/// # Syntax
///
/// ```rust,ignore
/// all_syntax_cases! {
///     setup => {
///         generated_fn_prefix: "prefix",
///         additional_input_type: YourType,
///         system_functions_test: false,  // Optional: default false
///     }
///     default_cases => {
///         // Functions called for all matching types
///         fn handler_name(param: &mut SynType, additional: AdditionalType);
///         
///         #[after_system]  // Optional: run after system traversal
///         fn late_handler(param: &mut SynType, additional: AdditionalType);
///     }
///     special_cases => {
///         // Functions called for specific syn variants
///         fn special_handler(variant: &mut syn::ExprTry, additional: AdditionalType);
///     }
/// }
/// ```
///
/// # Parameters
///
/// ## setup
///
/// - `generated_fn_prefix` - String prefix for all generated function names (e.g., `"handle"`
///   generates `handle_item`, `handle_expr`, etc.)
/// - `additional_input_type` - Type of additional context passed to all handlers. Can be any type
///   (reference, value, mutable reference). This type is passed through the entire traversal.
/// - `system_functions_test` - Optional boolean (default: `false`). When `true`, enables validation
///   that all system-generated functions are actually invoked during macro expansion. This helps detect
///   coverage gaps in the macro's traversal logic. Use this when developing or debugging the macro itself,
///   not in production code.
///
/// ## default_cases
///
/// Functions that handle any type matching their parameter signature. Handlers are automatically called
/// for matching fields, with smart unwrapping of `Box<T>`, `Vec<T>`, and `Punctuated<T, _>`. Note: `Option<T>`
/// fields are traversed via system-generated functions rather than direct smart unwrapping.
///
/// **Collection handling**:
/// - **2 parameters** (param + context): Iterates collections, calling handler per element
/// - **3+ parameters**: Passes entire collections, enabling multi-field correlation from same node
///
/// Mark with `#[after_system]` to run after traversing child nodes (for post-processing).
///
/// ## special_cases
///
/// Functions that handle specific syn type variants (like `syn::ExprTry`, `syn::ItemMod`). These
/// **completely override** default handlers for their variant—defaults won't run, and traversal stops
/// unless you explicitly call system handlers (e.g., `{prefix}_expr_handle(&mut expr, context)`).
///
/// Like `default_cases`, special case handlers also benefit from smart unwrapping of `Box<T>`,
/// `Vec<T>`, and `Punctuated<T, _>` when matching function parameters to struct fields.
///
/// # Generated Functions
///
/// The macro generates handler functions for all major syn types:
/// - `{prefix}_item_handle` - Handles `syn::Item` variants
/// - `{prefix}_expr_handle` - Handles `syn::Expr` variants
/// - `{prefix}_stmt_handle` - Handles `syn::Stmt` variants
/// - `{prefix}_pat_handle` - Handles `syn::Pat` variants
/// - `{prefix}_type_handle` - Handles `syn::Type` variants
/// - And many more for generics, blocks, signatures, fields, etc.
///
/// Each generated function contains exhaustive match arms for all variants of its type.
///
/// # Type Matching
///
/// The macro intelligently matches function parameters to struct fields:
///
/// - **Direct types**: `&mut syn::Expr` matches `expr: Expr`, `Box<Expr>`, `Option<Expr>`
/// - **Collections** (2-param handlers): `&mut syn::Expr` matches `Vec<Expr>` or `Punctuated<Expr, T>` (iterates per element)
/// - **Collections** (3+ param handlers): `&mut Vec<syn::Attribute>` matches as whole value (no iteration)
///
/// # Examples
///
/// ## Finding Function Calls (Special Cases)
///
/// ```rust,ignore
/// use all_syntax_cases::all_syntax_cases;
///
/// #[derive(Default)]
/// struct CallFinder { calls: Vec<String> }
///
/// all_syntax_cases! {
///     setup => {
///         generated_fn_prefix: "find",
///         additional_input_type: &mut CallFinder,
///     }
///     default_cases => {}
///     special_cases => {
///         fn handle_call(call: &mut syn::ExprCall, finder: &mut CallFinder);
///     }
/// }
///
/// fn handle_call(call: &mut syn::ExprCall, finder: &mut CallFinder) {
///     finder.calls.push(call.func.to_token_stream().to_string());
///     // Note: Traversal stops here unless we call find_expr_handle manually
/// }
/// ```
///
/// ## Multi-Field Correlation (3+ Parameters)
///
/// ```rust,ignore
/// all_syntax_cases! {
///     setup => {
///         generated_fn_prefix: "analyze",
///         additional_input_type: &mut Analysis,
///     }
///     default_cases => {
///         // Receives BOTH fields together from each node (called once per struct/enum)
///         fn analyze_attrs_and_generics(
///             attrs: &mut Vec<syn::Attribute>,  // Entire vector
///             generics: &mut syn::Generics,
///             analysis: &mut Analysis
///         );
///         
///         // 2 params: iterates and calls once per attribute across all nodes
///         fn analyze_attr(attr: &mut syn::Attribute, analysis: &mut Analysis);
///     }
///     special_cases => {}
/// }
/// ```
///
/// ## Post-Processing with `#[after_system]`
///
/// ```rust,ignore
/// all_syntax_cases! {
///     setup => {
///         generated_fn_prefix: "transform",
///         additional_input_type: &mut Context,
///     }
///     default_cases => {
///         fn pre_process(expr: &mut syn::Expr, ctx: &mut Context);
///         
///         #[after_system]  // Runs after children of Expr processed
///         fn post_process(expr: &mut syn::Expr, ctx: &mut Context);
///     }
///     special_cases => {}
/// }
/// ```
///
/// # Errors and Panics
///
/// Compile-time panics occur when:
/// - Required `setup` parameters (`generated_fn_prefix`, `additional_input_type`) are missing
/// - Handler functions never match any syntax node (signature doesn't match any fields)
/// - `additional_input_type` appears multiple times in a signature (must be distinct from syn types)
/// - `system_functions_test: true` enabled and internal functions not invoked (for macro debugging)
///
/// # Limitations
///
/// - **Incomplete coverage**: `TokenStream` fields (e.g., `syn::Macro::tokens`) are not traversed
/// - **Maintenance lag**: Manual updates needed when syn adds new syntax (use `system_functions_test` to detect gaps)
///
/// See comparison with `syn::visit_mut` below for complete coverage alternative.
///
/// # How It Works
///
/// 1. Generates handler functions for all syn types with exhaustive match arms
/// 2. For each variant: checks special cases first → if matched, calls only that handler and stops traversal
/// 3. If no special case: calls all matching default handlers → recursively traverses nested nodes
/// 4. Threads `additional_input` through all calls
///
/// # Use Cases
///
/// Syntax tree traversal • Code transformation • Pattern detection • Static analysis • Custom linting • Complex procedural macros
///
/// # Comparison with `syn::visit_mut`
///
/// Both traverse and mutate syn ASTs, but with different tradeoffs:
///
/// ## Advantages of `all_syntax_cases!`
///
/// - **Multi-field correlation**: Access multiple fields from same node in one handler (e.g., `attrs` + `generics` together)
/// - **Default vs. special cases**: Default handlers run for all types; special cases override and stop traversal
/// - **Automatic type matching**: Smart unwrapping of `Box<T>`, `Option<T>`, `Vec<T>`, `Punctuated<T, _>`
/// - **Pre/post-processing**: Use `#[after_system]` to run handlers after child traversal
///
/// ## Advantages of `syn::visit_mut`
///
/// - **Always current**: Auto-updates with syn releases; no manual maintenance
/// - **Established**: Well-documented, widely used, thoroughly tested
/// - **Simpler for single fields**: Straightforward trait-based approach
///
/// ## When to Use Which
///
/// **Use `all_syntax_cases!`** for multi-field correlation, default/special case separation, or pre/post-processing.
///
/// **Use `syn::visit_mut`** for stability, deep nesting, or single-field-at-a-time patterns.
///
/// # See Also
///
/// - [`syn::visit_mut`](https://docs.rs/syn/latest/syn/visit_mut/index.html) - Alternative visitor pattern in syn
/// - [`syn::fold`](https://docs.rs/syn/latest/syn/fold/index.html) - Transforming visitor pattern
pub fn all_syntax_cases(item: TokenStream) -> TokenStream {
    all_syntax_cases::all_syntax_cases(item)
}
#[proc_macro]
#[doc(hidden)]
pub fn all_syntax_cases_debug(item: TokenStream) -> TokenStream {
    let result = all_syntax_cases::all_syntax_cases(item);
    panic!("{}", result);
}
