use proc_macro2::{Group, TokenTree};
use quote::ToTokens;

pub fn never_any<T>() -> T {
    panic!("This function should never be called in runtime, used only for type checking")
}

///Add braces around TokenStream
pub fn braced(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    TokenTree::Group(Group::new(proc_macro2::Delimiter::Brace, item)).into()
}

pub fn iter_token_stream(items: impl Iterator<Item = impl ToTokens>) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();
    for item in items {
        output.extend(item.into_token_stream());
    }
    output
}
