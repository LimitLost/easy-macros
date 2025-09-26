use easy_macros_helpers_macro_safe::indexed_name;
use quote::quote;

fn main() {
    let field_names = indexed_name(syn::parse_quote!(field), 3);
    let output = quote! {
        struct MyStruct {
            #(#field_names: i32,)*
        }
    };
    // Generates: struct MyStruct { field0: i32, field1: i32, field2: i32, }
    
    println!("{}", output);
}