// Example showing how to use parse_macro_input! in a procedural macro
// Note: This is a demonstration - the actual usage would be in a proc macro crate

#[docify::export]
fn parse_macro_example() {
    // This would be used in a real procedural macro like:
    /*
    use easy_macros_helpers_macro_safe::parse_macro_input;
    use proc_macro::TokenStream;

    #[proc_macro]
    pub fn my_macro(input: TokenStream) -> anyhow::Result<TokenStream> {
        //This doesn't return TokenStream on compile errors, but Ok(TokenStream) with compile_error! inside
        let parsed = parse_macro_input!(input as syn::DeriveInput);

        // Process parsed input...
        Ok(quote! {
            // Generated code
        }.into())
    }
    */

    println!("This example demonstrates the parse_macro_input! macro pattern.");
    println!("See the source comments for the actual usage in procedural macros.");
}

fn main() {
    parse_macro_example();
}