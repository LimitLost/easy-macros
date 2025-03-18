mod matched_check;
mod matched_check_no_fields;
mod struct_check;

use proc_macro::TokenStream;

#[proc_macro]
///Macro used by all_syntax_cases
///
/// Uses `result_matches`, `default_functions`, `system_functions` and `special_functions`, without requesting them in macro input
pub fn matched_check(item: TokenStream) -> TokenStream {
    matched_check::matched_check(item)
}

#[proc_macro]
///Macro used by all_syntax_cases
///
/// Uses `result_matches`, `default_functions`, `system_functions` and `special_functions`, without requesting them in macro input
pub fn matched_check_no_fields(item: TokenStream) -> TokenStream {
    matched_check_no_fields::matched_check_no_fields(item)
}

#[proc_macro]
///Macro used by all_syntax_cases
///
/// Uses `result_matches`, `default_functions`, `system_functions` and `special_functions`, without requesting them in macro input
pub fn struct_check(item: TokenStream) -> TokenStream {
    struct_check::struct_check(item)
}
