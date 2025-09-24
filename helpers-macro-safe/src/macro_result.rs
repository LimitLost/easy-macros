use proc_macro2::TokenStream;

/// A builder for accumulating and formatting token streams in procedural macros.
///
/// `MacroResult` provides a convenient way to collect multiple token streams
/// and combine them into a single result. It's particularly useful when generating
/// code that consists of multiple statements or items that need to be grouped together.
///
/// # Examples
///
/// ```rust
/// use easy_macros_helpers_macro_safe::MacroResult;
/// use quote::quote;
///
/// let mut result = MacroResult::default();
///
/// // Add multiple token streams
/// result.add(quote! { let x = 1; });
/// result.add(quote! { let y = 2; });
/// result.add(quote! { println!("{} + {} = {}", x, y, x + y); });
///
/// // Wrap in braces to create a block
/// result.braced();
///
/// let tokens = result.finalize();
/// // Result: { let x = 1; let y = 2; println!("{} + {} = {}", x, y, x + y); }
/// ```
#[derive(Debug, Default)]
pub struct MacroResult {
    result: TokenStream,
}

impl MacroResult {
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
    /// ```rust
    /// use easy_macros_helpers_macro_safe::MacroResult;
    /// use quote::quote;
    ///
    /// let mut result = MacroResult::default();
    /// result.add(quote! { fn hello() });
    /// result.add(quote! { { println!("Hello, world!"); } });
    /// ```
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
    /// ```rust
    /// use easy_macros_helpers_macro_safe::MacroResult;
    /// use quote::quote;
    ///
    /// let mut result = MacroResult::default();
    /// result.add(quote! { let x = 42; });
    /// result.add(quote! { x * 2 });
    /// result.braced();
    ///
    /// let tokens = result.finalize();
    /// // Result: { let x = 42; x * 2 }
    /// ```
    pub fn braced(&mut self) {
        replace_with::replace_with_or_abort(&mut self.result, |result| {
            quote::quote! {
                {
                    #result
                }
            }
        });
    }

    /// Consumes the `MacroResult` and returns the final token stream.
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
    /// ```rust
    /// use easy_macros_helpers_macro_safe::MacroResult;
    /// use quote::quote;
    ///
    /// let mut result = MacroResult::default();
    /// result.add(quote! { println!("Done!"); });
    ///
    /// let final_tokens = result.finalize();
    /// ```
    pub fn finalize(self) -> TokenStream {
        self.result
    }
}
