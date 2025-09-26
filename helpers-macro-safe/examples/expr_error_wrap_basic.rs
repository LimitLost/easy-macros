use easy_macros_helpers_macro_safe::{expr_error_wrap, ErrorData};

fn main() {
    let mut errors = Vec::<String>::new();
    let mut expr = syn::parse_quote!(some_expression);

    // Add some errors
    errors.push("This field is required".to_string());
    errors.push("Invalid type specified".to_string());

    // Wrap expression with compile errors
    expr_error_wrap(&mut expr, &mut errors);
    // The expression now includes compile_error! calls
    
    println!("{}", quote::quote! { #expr });
}