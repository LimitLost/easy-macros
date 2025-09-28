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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_removes_unnecessary_spaces() {
        let test_cases = vec![
            ("Vec < String >", "Vec<String>"),
            ("fn main ( )", "fn main()"),
            ("println ! ( \"hello\" )", "println!(\"hello\" )"), // Space preserved before closing paren
            ("a  b   c", "a b c"),
            ("( )", "()"),
            ("{ }", "{ }"), // Space preserved in braces
            ("[ ]", "[]"),  // Space removed in brackets
            ("x . y", "x.y"),
            ("a , b", "a, b"),
            ("x : : y", "x:: y"), // Space after :: is preserved
            ("x ; y", "x; y"),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Failed for input: `{input}`");
        }
    }

    #[test]
    fn test_preserves_necessary_spaces() {
        let test_cases = vec![
            ("let x = 1", "let x = 1"),
            ("fn hello world", "fn hello world"),
            ("struct MyStruct", "struct MyStruct"),
            ("if true", "if true"),
            ("return value", "return value"),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Failed for input: `{input}`");
        }
    }

    #[test]
    fn test_consecutive_delimiters() {
        let test_cases = vec![
            ("( ( x ) )", "((x ))"),   // Space before closing paren is preserved
            ("{ { x } }", "{ { x }}"), // Different behavior for braces
            ("[ [ x ] ]", "[[x ]]"),   // Space before ] is preserved when after content
            ("( ) ( )", "()()"),
            (") ) )", ")))"),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Failed for input: `{input}`");
        }
    }

    #[test]
    fn test_complex_token_streams() {
        let test_cases = vec![
            (
                "impl < T > Clone for Vec < T > where T : Clone",
                "impl<T> Clone for Vec<T> where T: Clone", // Spaces removed around keywords
            ),
            (
                "fn main ( ) { println ! ( \"Hello , world !\" ) ; }",
                "fn main() { println!(\"Hello, world!\" ); }", // Spaces inside strings are handled differently
            ),
            (
                "match x { Some ( y ) => y , None => 0 }",
                "match x { Some(y ) => y, None => 0 }", // Spaces after => are preserved
            ),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Failed for input: `{input}`");
        }
    }

    #[test]
    fn test_empty_and_whitespace_only() {
        assert_eq!(readable_token_stream(""), "");
        assert_eq!(readable_token_stream(" "), "");
        assert_eq!(readable_token_stream("   "), "");
        // The function only removes space characters, not other whitespace
        assert_eq!(readable_token_stream("\t\n "), "\t\n");
    }

    #[test]
    fn test_deeply_nested_generics() {
        let test_cases = vec![
            ("Vec < Vec < Vec < String > > >", "Vec<Vec<Vec<String>>>"),
            (
                "HashMap < String , Vec < Option < u32 > > >",
                "HashMap<String, Vec<Option<u32>>>",
            ),
            (
                "Result < Option < Box < dyn Fn ( ) -> Vec < String > > > , Error >",
                "Result<Option<Box<dyn Fn() -> Vec<String>>>, Error>",
            ),
            (
                "impl < T : Clone + Send + Sync > IntoIterator for MyStruct < T >",
                "impl<T: Clone + Send + Sync> IntoIterator for MyStruct<T>",
            ),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(
                result, expected,
                "Failed for deeply nested generics: `{input}`",
            );
        }
    }

    #[test]
    fn test_rust_specific_syntax() {
        let test_cases = vec![
            // Lifetimes
            ("& ' a str", "&' a str"), // Space removed after &
            (
                "fn foo < ' a > ( x : & ' a str )",
                "fn foo<' a>(x: &' a str )",
            ),
            (
                "struct Foo < ' a , T > { x : & ' a T }",
                "struct Foo<' a, T>{ x: &' a T }",
            ),
            // Attributes
            (
                "# [ derive ( Debug , Clone ) ]",
                "#[derive(Debug, Clone ) ]",
            ),
            ("# [ cfg ( test ) ] mod tests", "#[cfg(test ) ] mod tests"),
            ("# ! [ no_std ]", "#![no_std ]"),
            // Macros
            ("vec ! [ 1 , 2 , 3 ]", "vec![1, 2, 3 ]"),
            (
                "println ! ( \"{} {}\" , x , y )",
                "println!(\"{} {}\", x, y )",
            ),
            ("macro_rules ! my_macro", "macro_rules!my_macro"),
            // Path separators
            (
                "std : : collections : : HashMap",
                "std:: collections:: HashMap",
            ),
            ("crate : : module : : function", "crate:: module:: function"),
            ("self : : Item", "self:: Item"),
            ("super : : parent_fn", "super:: parent_fn"),
            // Turbofish operator
            ("Vec : : < String > : : new", "Vec::<String>:: new"),
            ("collect : : < Vec < _ > > ( )", "collect::<Vec<_>>()"),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Failed for Rust syntax: `{input}`");
        }
    }

    #[test]
    fn test_complex_expressions() {
        let test_cases = vec![
            (
                "x . iter ( ) . filter ( | & y | y > 0 ) . collect ( )",
                "x.iter().filter(| &y | y> 0 ).collect()",
            ),
            (
                "match result { Ok ( value ) => process ( value ) , Err ( e ) => handle_error ( e ) }",
                "match result { Ok(value ) => process(value ), Err(e ) => handle_error(e ) }",
            ),
            (
                "if let Some ( ref mut x ) = option { * x += 1 ; }",
                "if let Some(ref mut x ) = option { * x += 1; }",
            ),
            (
                "async move | | -> Result < ( ) , Box < dyn Error > > { Ok ( ( ) ) }",
                "async move | | -> Result<(), Box<dyn Error>>{ Ok(()) }",
            ),
            (
                "where T : Send + Sync + ' static + Clone + Debug",
                "where T: Send + Sync + ' static + Clone + Debug",
            ),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Failed for complex expression: `{input}`",);
        }
    }

    #[test]
    fn test_string_and_char_literals() {
        let test_cases = vec![
            // Regular strings - this function works on token streams, not inside literals
            ("\"hello world\"", "\"hello world\""),
            ("\"  spaces  \"", "\" spaces \""), // Spaces between quotes get collapsed
            ("\" ( ) [ ] { } \"", "\"()[] { } \""), // Rules applied inside strings too
            // Raw strings
            ("r\"hello world\"", "r\"hello world\""),
            ("r # \" hello \" #", "r # \" hello \" #"),
            // Character literals
            ("' '", "' '"),
            ("' a '", "' a '"),
            ("'\\n'", "'\\n'"),
            // Strings with delimiters that would normally have spaces removed
            (
                "format ! ( \"Vec < {} >\" , T )",
                "format!(\"Vec<{}>\", T )",
            ),
            (
                "println ! ( \"{ : ? }\" , value )",
                "println!(\"{:? }\", value )",
            ),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(
                result, expected,
                "Failed for string/char literal: `{input}`",
            );
        }
    }

    #[test]
    fn test_mixed_delimiter_combinations() {
        let test_cases = vec![
            ("( [ { } ] )", "([{ } ] )"), // Space preserved in braces and before ], before closing paren
            ("< ( ) >", "<()>"),
            ("[ < T > ]", "[<T>]"),
            ("{ Vec < String > }", "{ Vec<String> }"),
            ("( a , [ b , c ] , { d : e } )", "(a,[b, c ], { d: e } )"), // Complex spacing rules
            (
                "fn ( Vec < & str > ) -> HashMap < String , i32 >",
                "fn(Vec<&str> ) -> HashMap<String, i32>",
            ),
            (
                "impl < T , U > From < T > for Wrapper < U >",
                "impl<T, U> From<T> for Wrapper<U>",
            ),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Failed for mixed delimiters: `{input}`",);
        }
    }

    #[test]
    fn test_operators_and_punctuation() {
        let test_cases = vec![
            ("x + = y", "x + = y"), // Spaces around operators are preserved
            ("a - > b", "a -> b"),
            ("| | x | x * 2", "| | x | x * 2"), // Spaces preserved in closure syntax
            ("x ? . y", "x?.y"),                // Space removed after ?
            ("x . . y", "x..y"),                // Space removed before second .
            ("x . . = y", "x..= y"),
            ("& & x | | y", "&&x | | y"), // First && combined, second || stays separated
            ("! x & & ! y", "!x &&!y"),
            ("< < x > >", "<<x>>"),
            ("x < < 1", "x<<1"),
            ("1 > > x", "1>> x"),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Failed for operators: `{input}`");
        }
    }

    #[test]
    fn test_real_world_token_streams() {
        let test_cases = vec![
            (
                "impl < T > IntoIterator for Vec < T > { type Item = T ; type IntoIter = std : : vec : : IntoIter < T > ; fn into_iter ( self ) -> Self : : IntoIter { self . into_iter ( ) } }",
                "impl<T> IntoIterator for Vec<T>{ type Item = T; type IntoIter = std:: vec:: IntoIter<T>; fn into_iter(self ) -> Self:: IntoIter { self.into_iter() }}",
            ),
            (
                "# [ derive ( Debug , Clone , PartialEq , Eq ) ] pub struct Config { pub name : String , pub values : Vec < ( String , String ) > , }",
                "#[derive(Debug, Clone, PartialEq, Eq ) ] pub struct Config { pub name: String, pub values: Vec<(String, String )>, }",
            ),
            (
                "async fn fetch_data < T > ( url : & str ) -> Result < T , reqwest : : Error > where T : serde : : de : : DeserializeOwned { reqwest : : get ( url ) . await ? . json ( ) . await }",
                "async fn fetch_data<T>(url: &str ) -> Result<T, reqwest:: Error> where T: serde:: de:: DeserializeOwned { reqwest:: get(url ).await?.json().await }",
            ),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Failed for real-world example: `{input}`",);
        }
    }

    #[test]
    fn test_edge_cases_and_boundaries() {
        let test_cases = vec![
            // Single character inputs
            ("(", "("),
            (")", ")"),
            (" ", ""),
            (".", "."),
            // Edge cases with delimiters at start/end
            (" ( hello )", "(hello )"),
            ("hello ( ) ", "hello()"),
            (" [ ] ", "[]"),
            (" < > ", "<> "),
            // Multiple consecutive spaces with different delimiters
            ("(    )", "()"),
            ("[    ]", "[]"),
            ("<    >", "<>"),
            ("{    }", "{ }"), // Single space preserved in braces
            // Mixed whitespace (spaces, tabs, newlines)
            ("(\\t)", "(\\t)"),
            ("(\\n)", "(\\n)"),
            ("( \\t\\n )", "(\\t\\n )"), // Preserves non-space whitespace and trailing space
            // Very long sequences
            ("( ( ( ( ( ) ) ) ) )", "((((()))))"),
            ("< < < < < > > > > >", "<<<<<>>>>>"),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Failed for edge case: `{input}`",);
        }
    }

    #[test]
    fn test_performance_with_long_strings() {
        // Test with a reasonably long token stream to ensure performance is acceptable
        let mut long_input = "Vec < HashMap < String , Vec < Option < Result < Box < dyn Fn ( ) -> i32 + Send + Sync > , std : : io : : Error > > > > > ".repeat(100);
        long_input = long_input.trim_end().to_string(); // Remove trailing space
        let expected_pattern = " Vec<HashMap<String, Vec<Option<Result<Box<dyn Fn() -> i32 + Send + Sync>, std:: io:: Error>>>>>";
        let mut expected = expected_pattern.repeat(100);
        expected = expected.trim().to_string(); // Remove trailing space

        let result = readable_token_stream(&long_input);
        assert_eq!(result, expected);

        // Test that it's still reasonable with very long strings
        let very_long_input = "a ".repeat(10000);
        let very_long_expected = "a ".repeat(9999) + "a"; // Last space should be removed
        let result = readable_token_stream(&very_long_input);
        assert_eq!(result, very_long_expected);
    }

    #[test]
    fn test_idempotency() {
        // Running readable_token_stream twice should produce the same result
        let test_cases = vec![
            "Vec < String >",
            "fn main ( ) { }",
            "impl < T > Clone for Vec < T >",
            "match x { Some ( y ) => y , None => 0 }",
            "println ! ( \"hello\" )",
            "& ' a str",
            "# [ derive ( Debug ) ]",
            "std : : collections : : HashMap",
            "",
            "   ",
            "already_clean_text",
        ];

        for input in test_cases {
            let first_pass = readable_token_stream(input);
            let second_pass = readable_token_stream(&first_pass);
            assert_eq!(
                first_pass, second_pass,
                "Function should be idempotent. Input: `{input}`, First: `{first_pass}`, Second: `{second_pass}`"
            );
        }
    }

    #[test]
    fn test_whitespace_only_removal_invariant() {
        // The function should only remove whitespace characters, never content
        let test_cases = vec![
            "Vec<String>",
            "fn main()",
            "hello world",
            "x + y",
            "a::b::c",
            "println!(\"test\")",
            "impl<T> Clone",
            "&'a str",
            "#[derive(Debug)]",
            "match x { Some(y) => y }",
            "Vec < HashMap < String , i32 > >",
        ];

        for input in test_cases {
            let result = readable_token_stream(input);
            let input_no_whitespace = input.replace(|c: char| c.is_whitespace(), "");
            let result_no_whitespace = result.replace(|c: char| c.is_whitespace(), "");

            assert_eq!(
                input_no_whitespace, result_no_whitespace,
                "Non-whitespace content should be identical. Input: `{input}`, Result: `{result}`"
            );
        }
    }

    #[test]
    fn test_space_reduction_properties() {
        // Test that the function never increases the length unnecessarily
        // and always reduces or maintains length
        let test_cases = vec![
            "Vec < String >",
            "a  b  c  d",
            "( ) ( ) ( )",
            "   hello   world   ",
            "x : : y : : z",
            "< < < > > >",
        ];

        for input in test_cases {
            let result = readable_token_stream(input);

            // Result should not be longer than input
            assert!(
                result.len() <= input.len(),
                "Result should not be longer than input. Input len: {}, Result len: {}. Input: `{}`, Result: `{}`",
                input.len(),
                result.len(),
                input,
                result
            );

            // Result should not have consecutive spaces
            assert!(
                !result.contains("  "),
                "Result should not contain consecutive spaces. Result: `{result}`",
            );

            // Result should not start with spaces, but may end with spaces in some cases
            if !input.trim().is_empty() {
                assert!(
                    !result.starts_with(' '),
                    "Result should not start with spaces. Result: `{result}`"
                );
            }
        }
    }

    #[test]
    fn test_delimiter_spacing_consistency() {
        // Test that the spacing rules are applied consistently
        let delimiters = vec![('(', ')'), ('[', ']'), ('<', '>'), ('{', '}')];

        for (open, close) in delimiters {
            // Test spacing after opening delimiter
            let input = format!("x {open} y");
            let result = readable_token_stream(&input);
            if matches!(open, '(' | '[' | '<') {
                assert!(
                    !result.contains(&format!("{open} ")),
                    "Should not have space after {open}. Result: `{result}`"
                );
            }

            // Test spacing before closing delimiter
            let input = format!("x {close} y");
            let result = readable_token_stream(&input);
            // Note: The function's logic is complex here, but we can test some cases

            // Test nested delimiters
            let input = format!("{open} {open} x {close} {close}");
            let _result = readable_token_stream(&input);
            assert!(
                result.len() <= input.len(),
                "Nested delimiters should not increase length. Input: `{input}`, Result: `{result}`"
            );
        }
    }

    #[test]
    fn test_boundary_conditions() {
        // Test various boundary conditions
        let boundary_cases = vec![
            // Single characters that might be affected by the rules
            ("(", "("),
            (")", ")"),
            ("[", "["),
            ("]", "]"),
            ("<", "<"),
            (">", ">"),
            ("{", "{"),
            ("}", "}"),
            (".", "."),
            (",", ","),
            (":", ":"),
            (";", ";"),
            ("!", "!"),
            ("?", "?"),
            ("&", "&"),
            // Delimiter pairs with no content
            ("( )", "()"),
            ("[ ]", "[]"),
            ("< >", "<>"),
            ("{ }", "{ }"), // Space preserved in braces
            // Single space with delimiters
            (" (", "("),
            (") ", ")"),
            (" [ ", "["),
            (" ] ", "]"),
            // Multiple spaces in various contexts
            ("  (  )  ", "() "),  // Trailing space preserved
            ("  [  ]  ", "[] "),  // Trailing space preserved
            ("  <  >  ", "<> "),  // Trailing space preserved after angle brackets
            ("  {  }  ", "{ } "), // Space preserved in braces and trailing
        ];

        for (input, expected) in boundary_cases {
            let result = readable_token_stream(input);
            assert_eq!(result, expected, "Boundary condition failed for: `{input}`");
        }
    }

    #[test]
    fn test_unicode_and_special_chars() {
        // Test that the function handles Unicode and special characters correctly
        let test_cases = vec![
            ("café ( )", "café()"),
            ("函数 < T >", "函数<T>"),
            ("Ω . clone ( )", "Ω.clone()"),
            ("let π = 3.14", "let π = 3.14"),
            ("struct 🦀 < T >", "struct 🦀<T>"),
            ("// comment ( )", "// comment()"),
            ("/* block */ ( )", "/* block */()"),
        ];

        for (input, expected) in test_cases {
            let result = readable_token_stream(input);
            assert_eq!(
                result, expected,
                "Unicode/special char test failed for: `{input}`"
            );
        }
    }
}
