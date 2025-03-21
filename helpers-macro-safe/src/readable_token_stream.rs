/// Removes all unnecessary spaces from string representation of token stream,
/// But not every space is unnecessary
///
/// Just removes spaces, that's it
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
                    ('(' | '!' | '&' | '[' | '<' | '>' | '.', _)
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
    assert_eq!(
        result.replace(|c: char| c.is_whitespace(), ""),
        tokens_str.replace(|c: char| c.is_whitespace(), ""),
        "Only whitespace should be removed from token stream | Result: `{}` | Original: `{}`",
        result,
        tokens_str
    );

    result
}
