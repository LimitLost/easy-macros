use proc_macro2::TokenStream;
#[derive(Debug, Default)]
pub struct MacroResult {
    result: TokenStream,
}

impl MacroResult {
    pub fn add(&mut self, item: TokenStream) {
        self.result.extend(item);
    }

    pub fn finalize(self) -> TokenStream {
        self.result
    }
}
