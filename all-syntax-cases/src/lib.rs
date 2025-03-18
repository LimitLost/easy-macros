use proc_macro::TokenStream;

mod all_syntax_cases;
mod helpers;

#[proc_macro]
pub fn all_syntax_cases(item: TokenStream) -> TokenStream {
    all_syntax_cases::all_syntax_cases(item)
}
