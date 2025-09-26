use easy_macros_helpers_macro_safe::MacroResult;
use quote::quote;

#[docify::export]
fn detailed_example() {
    let mut result = MacroResult::default();

    // Add multiple token streams
    result.add(quote! { let x = 1; });
    result.add(quote! { let y = 2; });
    result.add(quote! { println!("{} + {} = {}", x, y, x + y); });

    // Wrap in braces to create a block
    result.braced();

    let tokens = result.finalize();
    // Result: { let x = 1; let y = 2; println!("{} + {} = {}", x, y, x + y); }
    
    println!("{}", tokens);
}

fn main() {
    detailed_example();
}