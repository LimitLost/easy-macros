use easy_macros_helpers_macro_safe::MacroResult;
use quote::quote;

#[docify::export]
fn braced_example() {
    let mut result = MacroResult::default();
    result.add(quote! { let x = 42; });
    result.add(quote! { x * 2 });
    result.braced();

    let tokens = result.finalize();
    // Result: { let x = 42; x * 2 }
    
    println!("{}", tokens);
}

fn main() {
    braced_example();
}