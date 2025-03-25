use proc_macro2::TokenStream;
#[derive(Debug, Default)]
pub struct MacroResult {
    result: TokenStream,
}

impl MacroResult {
    pub fn add(&mut self, item: TokenStream) {
        self.result.extend(item);
    }
    ///Wraps result with a pair of braces
    pub fn braced(&mut self) {
        replace_with::replace_with_or_abort(&mut self.result, |result| {
            quote::quote! {
                {
                    #result
                }
            }
        });
    }

    pub fn finalize(self) -> TokenStream {
        self.result
    }
}
