use proc_macro2::{Delimiter, TokenStream, TokenTree};

/// Removes spaces between tokens (when using `TokenStream::to_string()` sometimes spaces appear, sometimes they don't (depending on the generation context))
///
/// When operating on `syn::File` they generally appear
///
/// But inside of procedural macros they generally don't
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
