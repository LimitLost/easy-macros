use easy_macros_helpers_macro_safe::ErrorData;

#[docify::export]
fn error_data_basic() {
    let mut errors = Vec::<String>::new();
    errors.push("Invalid syntax".to_string());
    errors.push("Missing required field".to_string());

    assert!(!errors.no_errors());
    let error_messages = errors.error_data();
    assert!(errors.no_errors()); // Errors are consumed
    
    println!("Error messages: {:?}", error_messages);
}

fn main() {
    error_data_basic();
}