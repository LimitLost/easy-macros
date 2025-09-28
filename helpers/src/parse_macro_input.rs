#[macro_export]
/// Adjusted version of syn's `parse_macro_input!` macro for `#[anyhow_result]` procedural macro attribute.
///
/// Unlike syn's original `parse_macro_input!` macro, this version returns `Ok(TokenStream)`
/// on parse errors instead of returning a `TokenStream` directly. This makes
/// it more suitable for use in procedural macros which are returning `anyhow::Result<TokenStream>`. See `anyhow_result` macro.
///
/// # Behavior
///
/// On successful parsing, the macro returns the parsed value directly.
/// On parse errors, it returns `Ok(TokenStream)` containing the error as `compile_error!` tokens,
/// allowing the error to be displayed at compile time while still providing a valid return value.
///
/// # Syntax
///
/// ```ignore
/// // Parse as a specific type
/// let input = parse_macro_input!(tokens as DeriveInput);
///
/// // Parse with a custom parser
/// let input = parse_macro_input!(tokens with syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated);
///
/// // Parse with type inference
/// let input = parse_macro_input!(tokens);
/// ```
///
/// # Examples
///
/// ```ignore
/// use easy_macros::{parse_macro_input, anyhow_result};
/// use proc_macro::TokenStream;
/// use syn::DeriveInput;
///
/// #[proc_macro_derive(MyDerive)]
/// #[anyhow_result]
/// pub fn my_derive(input: TokenStream) -> anyhow::Result<TokenStream> {
///     // This will return compile errors automatically on parse failure
///     let input = parse_macro_input!(input as DeriveInput);
///     
///     // Your macro logic here...
///     Ok(quote::quote! {
///         // Generated code
///     }.into())
/// }
/// ```
///
/// # Advantages over syn's version
///
/// - Returns `Ok(TokenStream)` on errors instead of raw `TokenStream`
/// - Better integration with macros that return for example `anyhow::Result<TokenStream>`
///
/// # Parameters
///
/// - `$tokenstream` - The input `TokenStream` to parse
/// - `$ty` - The target type to parse into (with `as` syntax)
/// - `$parser` - A custom parser function (with `with` syntax)
macro_rules! parse_macro_input {
    ($tokenstream:ident as $ty:ty) => {
        match syn::parse::<$ty>($tokenstream) {
            syn::__private::Ok(data) => data,
            syn::__private::Err(err) => {
                return Ok(syn::__private::TokenStream::from(err.to_compile_error()));
            }
        }
    };
    ($tokenstream:ident with $parser:path) => {
        match syn::parse::Parser::parse($parser, $tokenstream) {
            syn::__private::Ok(data) => data,
            syn::__private::Err(err) => {
                return Ok(syn::__private::TokenStream::from(err.to_compile_error()));
            }
        }
    };
    ($tokenstream:ident) => {
        $crate::parse_macro_input!($tokenstream as _)
    };
}
