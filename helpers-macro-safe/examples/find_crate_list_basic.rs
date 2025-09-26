use easy_macros_helpers_macro_safe::find_crate_list;
use quote::quote;

#[docify::export]
fn find_crate_list_example() {
    // Try multiple crates with fallbacks
    let async_crates = &[
        ("tokio", quote!(::runtime::Runtime)),
        ("async-std", quote!(::task)),
        ("smol", quote!()),
    ];

    if let Some(async_path) = find_crate_list(async_crates) {
        // Uses first available async runtime
        println!("Found async runtime: {}", async_path);
    }

    // Try serialization libraries
    let serde_crates = &[
        ("serde_json", quote!(::to_string)),
        ("ron", quote!(::to_string)),
        ("bincode", quote!(::serialize)),
    ];

    if let Some(serde_path) = find_crate_list(serde_crates) {
        println!("Found serialization library: {}", serde_path);
    }
}

fn main() {
    find_crate_list_example();
}