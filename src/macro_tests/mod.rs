use all_syntax_cases::all_syntax_cases;
use proc_macro_tests::macro_test_eq;

#[macro_test_eq]
struct TestStruct {
    field: i32,
}
//Tests if all default functions are used and alright (no error making mistakes inside of them)
struct Nothing;
all_syntax_cases! {
    setup => {
        generated_fn_prefix: "example",
        additional_input_type: Nothing,
        system_functions_test: true
    }
    default_cases => {}
    special_cases => {}
}
