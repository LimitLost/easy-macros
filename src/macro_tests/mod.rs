use proc_macro_tests::macro_test_eq;

#[macro_test_eq]
struct TestStruct {
    field: i32,
}
