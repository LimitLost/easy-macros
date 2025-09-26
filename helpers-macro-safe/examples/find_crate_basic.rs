use easy_macros_helpers_macro_safe::find_crate;
use quote::quote;

#[docify::export]
fn find_crate_example() {
    // Find a crate without additional path
    if let Some(path) = find_crate("serde", quote!()) {
        // Returns: `serde` or `crate` depending on context
        println!("Found serde: {}", path);
    }

    // Find a crate with additional path segments
    if let Some(path) = find_crate("quote", quote!(::quote)) {
        // Returns: `quote::quote` or `crate::quote`
        println!("Found quote::quote: {}", path);
    }

    // With a renamed crate in Cargo.toml:
    // [dependencies]
    // serde_renamed = { package = "serde", version = "1.0" }
    if let Some(path) = find_crate("serde", quote!(::Serialize)) {
        // Returns: `serde_renamed::Serialize`
        println!("Found serde::Serialize: {}", path);
    }
}

fn main() {
    find_crate_example();
}