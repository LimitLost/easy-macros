//Example with input
/*
// Only Token![] calls are supported, calling by specific Token Type is not
all_syntax_cases!{
    setup => {
        entry_ty: syn::Item,
        fn_name_prefix: "example_",
        additional_input_ty: Option<NoContext>,
    }
    default_cases => {
        fn example_default(attrs: &mut Vec<Attribute>, no_context: &mut Option<NoContext>);
    }
    special_cases => {
        //Mutable data request should not be allowed here
        fn example_try(expr_try: &mut syn::ExprTry, no_context: Option<NoContext>) ;
        fn example_macro(expr_macro: &mut syn::ExprMacro, no_context: Option<NoContext>) ;
    }

}
 */

mod data;
mod search;

use proc_macro::TokenStream;

//TODO Handle items
//TODO Handle expressions
//TODO Handle Generics (some have expr inside of them)

//TODO Create a list of every type found that can be used in default or special case (while computing this macro) (maybe?)
//TODO If passed in default or special function are never called show an error

///Creates a function covering all cases of provided type
/// additional_input is passed in deeper as a copy, not a mutable reference
/// Every item in for example block has it's own copy of additional_input
pub fn all_syntax_cases(item: TokenStream) -> TokenStream {}
