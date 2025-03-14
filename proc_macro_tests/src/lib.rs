use helpers::MacroResult;
use macros::has_attributes;
use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

#[proc_macro_derive(DeriveTestStruct)]
pub fn macro_test_struct(item: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(item as syn::ItemStruct);

    let mut result = MacroResult::new();

    if !has_attributes!(parsed, #[lol]) {
        //Show Compiler error
        result.add(quote! {
            compile_error!("#[lol] attribute on struct not found");
        });
    }

    result.finalize().into()
}

//TODO get_attribute

///Expected Struct:
/// struct TestStruct{
///     field: i32,
/// }
#[proc_macro_attribute]
pub fn macro_test_eq(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(item as syn::ItemStruct);

    let mut result = MacroResult::new();

    let not_real_struct: ItemStruct = syn::parse_quote! {
        struct TestStruct{
            field: i32,
        }
    };

    if parsed != not_real_struct {
        //Show Compiler error
        result.add(quote! {
            compile_error!("Structs are not equal");
        });
    }

    result.finalize().into()
}
