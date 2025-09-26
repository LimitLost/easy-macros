use easy_macros_helpers_macro_safe::{expr_error_wrap, ErrorData};
use syn::parse_quote;

#[docify::export]
fn expr_error_wrap_basic() {
    let mut expr = parse_quote!(42);
    let mut errors = vec![
        "This is a warning".to_string(),
        "Another issue found".to_string(),
    ];

    expr_error_wrap(&mut expr, &mut errors);

    // The expression is now wrapped with compile errors:
    // {
    //     compile_error!("This is a warning");
    //     compile_error!("Another issue found");
    //     42
    // }
    
    println!("{}", quote::quote! { #expr });
}

fn main() {
    expr_error_wrap_basic();
}