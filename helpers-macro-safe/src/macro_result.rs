use proc_macro2::TokenStream;

pub struct MacroResult {
    result: TokenStream,
}

impl MacroResult {
    pub fn new() -> Self {
        Self {
            result: TokenStream::new(),
        }
    }

    pub fn add(&mut self, item: TokenStream) {
        self.result.extend(item);
    }

    pub fn finalize(self) -> TokenStream {
        self.result
    }
}
