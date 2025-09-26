use easy_macros_helpers_macro_safe::MacroResult;
use quote::quote;

#[docify::export]
fn finalize_example() {
    let mut result = MacroResult::default();
    result.add(quote! { println!("Done!"); });

    let final_tokens = result.finalize();
    
    println!("{}", final_tokens);
}

fn main() {
    finalize_example();
}