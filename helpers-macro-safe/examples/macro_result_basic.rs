use easy_macros_helpers_macro_safe::MacroResult;
use quote::quote;

fn main() {
    let mut result = MacroResult::default();

    // Add multiple token streams
    result.add(quote! {
        println!("Hello");
    });
    result.add(quote! {
        println!("World");
    });

    // Wrap everything in braces
    result.braced();

    // Get final result
    let tokens = result.finalize();
    // Result: { println!("Hello"); println!("World"); }
    
    println!("{}", tokens);
}