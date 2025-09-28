//! Tests specifically for the context macro

use crate::context;
use anyhow::Context;

#[test]
fn context_basic_usage() {
    // Basic usage with no arguments - just file:line
    let ctx = context!();
    let result = ctx();

    // Should match exact format: "src/tests/context.rs:line"
    assert_eq!(result, format!("src/tests/context.rs:{}", line!() - 4));
}

#[test]
fn context_with_message() {
    // Usage with a simple message
    let ctx = context!("Operation failed");
    let result = ctx();

    // Should match exact format: "src/tests/context.rs:line\r\nOperation failed"
    assert_eq!(
        result,
        format!("src/tests/context.rs:{}\r\nOperation failed", line!() - 6)
    );
}

#[test]
fn context_with_formatting() {
    // Usage with format arguments
    let user_id = 42u64;
    let operation = "delete";

    let ctx = context!("Failed to {} user {}", operation, user_id);
    let result = ctx();

    // Should match exact format: "src/tests/context.rs:line\r\nFailed to delete user 42"
    assert_eq!(
        result,
        format!(
            "src/tests/context.rs:{}\r\nFailed to delete user 42",
            line!() - 8
        )
    );
}

#[test]
fn context_with_anyhow() {
    use std::fs;

    fn read_config() -> anyhow::Result<String> {
        fs::read_to_string("nonexistent.txt").with_context(context!("Failed to read configuration"))
    }

    // Test that the context is applied correctly
    let result = read_config();
    assert!(result.is_err());

    let error_string = format!("{:?}", result.unwrap_err());
    assert!(
        error_string.contains(
            format!(
                "src/tests/context.rs:{}\r\nFailed to read configuration",
                line!() - 12
            )
            .as_str()
        )
    );
}

#[test]
fn context_multiple_format_args() {
    let file_path = "/path/to/file.txt";
    let line_number = 142;
    let column = 5;

    let ctx = context!(
        "Parse error at {}:{}:{} - unexpected token",
        file_path,
        line_number,
        column
    );
    let result = ctx();

    // Should match exact format: "src/tests/context.rs:line\r\nParse error at /path/to/file.txt:142:5 - unexpected token"
    assert_eq!(
        result,
        format!(
            "src/tests/context.rs:{}\r\nParse error at /path/to/file.txt:142:5 - unexpected token",
            line!() - 13
        )
    );
}
