use proc_macro2::TokenStream;

/// A builder for accumulating and formatting token streams in procedural macros.
///
/// `TokensBuilder` provides a convenient way to collect multiple token streams
/// and combine them into a single result. It's particularly useful when generating
/// code that consists of multiple statements or items that need to be grouped together.
///
/// # Examples
///
#[doc = docify::embed!("src/examples.rs", tokens_builder_basic_usage)]
#[derive(Debug, Default)]
pub struct TokensBuilder {
    result: TokenStream,
}

impl TokensBuilder {
    /// Adds a token stream to the accumulated result.
    ///
    /// The new tokens are appended to the existing token stream.
    /// This method can be called multiple times to build up complex token sequences.
    ///
    /// # Arguments
    ///
    /// * `item` - The token stream to add to the result
    ///
    /// # Examples
    ///
    #[doc = docify::embed!("src/examples.rs", tokens_builder_add_example)]
    pub fn add(&mut self, item: TokenStream) {
        self.result.extend(item);
    }

    /// Wraps the accumulated result with a pair of braces, creating a block expression.
    ///
    /// This is useful when you want to group multiple statements or expressions
    /// into a single block that can be used as an expression or statement.
    ///
    /// # Examples
    ///
    #[doc = docify::embed!("src/examples.rs", tokens_builder_braced_example)]
    pub fn braced(&mut self) {
        replace_with::replace_with_or_abort(&mut self.result, |result| {
            quote::quote! {
                {
                    #result
                }
            }
        });
    }

    /// Consumes the `TokensBuilder` and returns the final token stream.
    ///
    /// This method should be called once you've finished building your result
    /// and are ready to return it from your function procedural macro.
    ///
    /// # Returns
    ///
    /// The accumulated token stream
    ///
    /// # Examples
    ///
    #[doc = docify::embed!("src/examples.rs", tokens_builder_finalize_example)]
    pub fn finalize(self) -> TokenStream {
        self.result
    }
}
