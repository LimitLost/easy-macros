mod matched_check;

use proc_macro::TokenStream;

#[proc_macro]
///Macro used by all_syntax_cases
pub fn matched_check(item: TokenStream) -> TokenStream {
    matched_check::matched_check(item)
}
