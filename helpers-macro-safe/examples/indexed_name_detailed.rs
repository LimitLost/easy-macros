use easy_macros_helpers_macro_safe::indexed_name;
use quote::quote;

#[docify::export]
fn indexed_name_example() {
    let base = syn::parse_quote!(field);
    let names = indexed_name(base, 3);

    // Use in a quote! macro to generate struct fields
    let output = quote! {
        struct MyStruct {
            #(#names: i32,)*
        }
    };
    // Expands to: struct MyStruct { field0: i32, field1: i32, field2: i32, }
    
    println!("{}", output);
}

fn main() {
    indexed_name_example();
}