//! Using this library inside of procedural macros should not cause any dependency loops, since this crate does not depend on any procedural macros.

mod indexed_name;
pub use indexed_name::indexed_name;

mod macro_result;
pub use macro_result::MacroResult;

mod expr_error_wrap;
pub use expr_error_wrap::{ErrorData, expr_error_wrap};

mod readable_token_stream;
pub use readable_token_stream::readable_token_stream;

#[macro_export]
///Original Syn macro doesn't return Ok(TokenStream) on error, but instead returns TokenStream
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
