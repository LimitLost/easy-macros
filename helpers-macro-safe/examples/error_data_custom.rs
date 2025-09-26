use easy_macros_helpers_macro_safe::ErrorData;

#[derive(Default)]
struct ValidationErrors {
    errors: Vec<String>,
    other_data: String,
}

impl ValidationErrors {
    fn add_error(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
    }
}

impl ErrorData for ValidationErrors {
    fn no_errors(&self) -> bool {
        self.errors.no_errors()
    }

    fn error_data(&mut self) -> Vec<String> {
        self.errors.error_data()
    }
}

#[docify::export]
fn error_data_custom() {
    let mut validator = ValidationErrors::default();
    validator.add_error("Test error");
    
    assert!(!validator.no_errors());
    let messages = validator.error_data();
    assert!(validator.no_errors());
    
    println!("Custom validation errors: {:?}", messages);
}

fn main() {
    error_data_custom();
}