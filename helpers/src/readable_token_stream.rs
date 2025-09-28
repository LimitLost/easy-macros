/// Formats a token stream string by removing unnecessary whitespace while preserving readability.
///
/// This function processes the string representation of token streams to remove
/// redundant spaces while keeping necessary spacing for readability. It's particularly
/// useful for cleaning up generated code or preparing token streams for debugging.
///
/// # Arguments
///
/// * `tokens_str` - A string representation of tokens (e.g., from `TokenStream :: to_string ()`)
///
/// # Returns
///
/// A cleaned string with unnecessary whitespace removed but readability preserved
///
/// # Whitespace Rules
///
/// The function removes spaces in these cases:
/// - Multiple consecutive spaces are collapsed to one
/// - Spaces after opening delimiters: `(`, `!`, `&`, `[`, `<`, `>`, `.`
/// - Spaces before closing delimiters and punctuation: `.`, `,`, `(`, `[`, `:`, `;`, `!`, `<`, `>`, `?`
/// - Spaces between consecutive closing delimiters: `))`, `}}`, `]]`
///
/// # Examples
///
#[doc = docify::embed!("src/examples.rs", readable_token_stream_example)]
/// # Use Cases
///
/// - Cleaning up token streams for debugging output
/// - Formatting generated code for better readability
/// - Preparing code for display in error messages
///
/// # Safety
///
/// This function includes an assertion to ensure that only whitespace is removed,
/// not actual token content. If this assertion fails, it indicates a bug in the
/// whitespace removal logic.
pub fn readable_token_stream(tokens_str: &str) -> String {
    let mut result = String::new();

    let mut char_iter_future = tokens_str.chars();
    char_iter_future.next();

    let char_iter_current = tokens_str.chars();

    let char_iter_future = char_iter_future.map(Some).chain(std::iter::once(None));

    let iters_zipped = char_iter_current.zip(char_iter_future);

    let mut last_char = ' ';

    for (c, future_c) in iters_zipped {
        match c {
            ' ' => {
                if last_char == ' ' {
                    continue;
                }
                match (last_char, future_c) {
                    ('>', Some('>' | '(' | '{' | '[' | ',' | ']' | ':' | ';')) => {
                        continue;
                    }
                    ('>', _) => {
                        result.push(c);
                        last_char = c;
                    }
                    ('(' | '!' | '&' | '[' | '<' | '.', _)
                    | (_, None | Some('.' | ',' | '(' | '[' | ':' | ';' | '!' | '<' | '>' | '?'))
                    | (')', Some(')'))
                    | ('}', Some('}'))
                    | (']', Some(']')) => {
                        continue;
                    }
                    _ => {
                        result.push(' ');
                        last_char = ' ';
                    }
                }
            }
            _ => {
                result.push(c);
                last_char = c;
            }
        }
    }

    //Test if we only removed whitespace
    #[cfg(test)]
    assert_eq!(
        result.replace(|c: char| c.is_whitespace(), ""),
        tokens_str.replace(|c: char| c.is_whitespace(), ""),
        "Only whitespace should be removed from token stream | Result: `{result}` | Original: `{tokens_str}`"
    );

    result
}
