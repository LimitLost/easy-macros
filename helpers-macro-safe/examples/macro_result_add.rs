use easy_macros_helpers_macro_safe::MacroResult;
use quote::quote;

#[docify::export]
fn add_example() {
    let mut result = MacroResult::default();
    result.add(quote! { fn hello() });
    result.add(quote! { { println!("Hello, world!"); } });

    let final_tokens = result.finalize();
    
    println!("{}", final_tokens);
}

fn main() {
    add_example();
}