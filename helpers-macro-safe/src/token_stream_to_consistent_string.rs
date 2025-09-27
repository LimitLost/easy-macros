use proc_macro2::{Delimiter, TokenStream, TokenTree};

/// Converts a token stream to a consistent string representation without spaces.
///
/// The standard `TokenStream::to_string()` method can produce inconsistent output
/// depending on the context - sometimes including spaces between tokens, sometimes not.
/// This function ensures a consistent, space-free representation that's reliable
/// for comparisons, hashing, or other operations requiring deterministic output.
///
/// # Context-Dependent Behavior of `TokenStream::to_string()`
///
/// - When operating on `syn::File`: spaces generally appear between tokens
/// - Inside procedural macros: spaces generally don't appear
/// - Other contexts: behavior may vary
///
/// # Arguments
///
/// * `tokens` - The token stream to convert to a consistent string
///
/// # Returns
///
/// A string representation with no spaces between tokens, ensuring consistent
/// output regardless of the original context
///
/// # Examples
///
#[doc = docify::embed!("src/examples.rs", token_stream_consistent_string_example)]
///
/// # Token Processing
///
/// The function handles different token types:
/// - **Groups**: Processes delimiters (`()`, `{}`, `[]`) and recursively processes contents
/// - **Identifiers**: Removes leading/trailing whitespace
/// - **Punctuation**: Removes leading/trailing whitespace  
/// - **Literals**: Removes leading/trailing whitespace
///
/// # Use Cases
///
/// - Comparing token streams for equality regardless of spacing
/// - Debugging token streams with predictable output
pub fn token_stream_to_consistent_string(tokens: TokenStream) -> String {
    let mut result_str = String::new();

    for token in tokens.into_iter() {
        match token {
            TokenTree::Group(group) => {
                match group.delimiter() {
                    Delimiter::Parenthesis => {
                        result_str.push('(');
                    }
                    Delimiter::Brace => {
                        result_str.push('{');
                    }
                    Delimiter::Bracket => {
                        result_str.push('[');
                    }
                    Delimiter::None => {}
                }
                result_str.push_str(&token_stream_to_consistent_string(group.stream()));
                match group.delimiter() {
                    Delimiter::Parenthesis => {
                        result_str.push(')');
                    }
                    Delimiter::Brace => {
                        result_str.push('}');
                    }
                    Delimiter::Bracket => {
                        result_str.push(']');
                    }
                    Delimiter::None => {}
                }
            }
            TokenTree::Ident(ident) => {
                result_str.push_str(&ident.to_string().trim_start().trim_end());
            }
            TokenTree::Punct(punct) => {
                result_str.push_str(&punct.to_string().trim_start().trim_end());
            }
            TokenTree::Literal(literal) => {
                result_str.push_str(&literal.to_string().trim_start().trim_end());
            }
        }
    }

    result_str
}
