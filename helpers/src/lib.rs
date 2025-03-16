mod indexed_name;
pub use indexed_name::indexed_name;

mod macro_result;
pub use macro_result::MacroResult;

pub use macros2::context_internal;

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

#[macro_export]
/// Same syntax as format! macro (from std)
///
/// Makes .with_context() from anyhow more convenient
///
/// Returns a closure
///
/// Add current file and line number to context
macro_rules! context {
    ($($arg:tt)*) => {
        ||{
            //Adds syntax checking from format! macro
            let _= ||{
                let _ = format!($($arg)*);
            };
            $crate::context_internal!($($arg)*);
        }
    };
}
