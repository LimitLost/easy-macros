use all_syntax_cases::all_syntax_cases;
use proc_macro_tests::{DeriveTestStruct, macro_test_eq};
use quote::ToTokens;

#[macro_test_eq]
struct TestStruct {
    field: i32,
}
//Tests if all default functions are used and alright (no error making mistakes inside of them)
#[derive(Debug, Clone)]
struct _Nothing {
    x: i32,
}

all_syntax_cases! {
    setup => {
        generated_fn_prefix: "example",
        additional_input_type: _Nothing,
        //Used for debugging
        system_functions_test: true
    }
    default_cases => {}
    special_cases => {}
}

#[derive(DeriveTestStruct)]
#[lol]
#[lmao]
#[xlold]
#[xdedd]
#[xnoned]
#[xb = "cd"]
#[bbb((lolspecialX))]
#[bbb((lol=X))]
#[bbb((lol = X))]
#[bbb((lol$$$X))]
#[bbb((lol((lul))X))]
#[sql(table = spec)]
struct _AttributeTest {}
