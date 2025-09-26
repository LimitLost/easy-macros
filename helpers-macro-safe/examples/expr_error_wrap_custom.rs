use easy_macros_helpers_macro_safe::{expr_error_wrap, ErrorData};
use syn::parse_quote;

#[derive(Default)]
struct MacroValidator {
    errors: Vec<String>,
    context: String,
}

impl MacroValidator {
    fn new(context: &str) -> Self {
        Self {
            errors: Vec::new(),
            context: context.to_string(),
        }
    }
    
    fn validate_field(&mut self, field_name: &str, is_valid: bool) {
        if !is_valid {
            self.errors.push(format!("Invalid field '{}' in {}", field_name, self.context));
        }
    }
}

impl ErrorData for MacroValidator {
    fn no_errors(&self) -> bool {
        self.errors.no_errors()
    }

    fn error_data(&mut self) -> Vec<String> {
        self.errors.error_data()
    }
}

#[docify::export]
fn expr_error_wrap_custom() {
    // Usage in a procedural macro context
    let mut validator = MacroValidator::new("MyStruct");
    validator.validate_field("name", false);
    validator.validate_field("id", false);

    let mut result_expr = parse_quote!(MyStruct::default());
    expr_error_wrap(&mut result_expr, &mut validator);

    // The expression now includes compile-time validation errors
    println!("{}", quote::quote! { #result_expr });
}

fn main() {
    expr_error_wrap_custom();
}