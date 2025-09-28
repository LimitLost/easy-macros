//! Tests specifically for the context macro

use crate::context;
use anyhow::Context;

#[test]
fn context_basic_usage() {
    // Basic usage with no arguments - just file:line
    let ctx = context!();
    let result = ctx();

    // Should match exact format: "src/tests/context.rs:line"
    assert!(result.starts_with("src/tests/context.rs:"));
    // Should not contain \r\n since it's just file:line
    assert!(!result.contains("\r\n"));
    // Should end with a line number (digits)
    assert!(result.ends_with((line!() - 8).to_string().as_str()));
}

#[test]
fn context_with_message() {
    // Usage with a simple message
    let ctx = context!("Operation failed");
    let result = ctx();

    // Should match exact format: "src/tests/context.rs:line\r\nOperation failed"
    assert!(result.starts_with("src/tests/context.rs:"));
    assert!(result.ends_with("\r\nOperation failed"));
    assert!(
        result
            .trim_end_matches("\r\nOperation failed")
            .ends_with((line!() - 9).to_string().as_str(),),
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
    assert!(result.starts_with("src/tests/context.rs:"));
    assert!(result.ends_with("\r\nFailed to delete user 42"));
    // Should have exactly one \r\n separator
    assert_eq!(result.matches("\r\n").count(), 1);
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
    assert!(error_string.contains("Failed to read configuration"));
    assert!(error_string.contains("context.rs"));
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
    assert!(result.starts_with("src/tests/context.rs:"));
    let expected_message = "Parse error at /path/to/file.txt:142:5 - unexpected token";
    assert!(result.ends_with(&format!("\r\n{}", expected_message)));
    // Should have exactly one \r\n separator
    assert_eq!(result.matches("\r\n").count(), 1);
}

#[test]
fn context_closure_reuse() {
    // Context closures can be stored and reused
    let error_context = context!("Database connection failed");

    // Use it multiple times
    let ctx1 = error_context();
    let ctx2 = error_context();

    // Both should produce the same context string
    assert_eq!(ctx1, ctx2);
    // Should match exact format: "src/tests/context.rs:line\r\nDatabase connection failed"
    assert!(ctx1.starts_with("src/tests/context.rs:"));
    assert!(ctx1.ends_with("\r\nDatabase connection failed"));
    // Should have exactly one \r\n separator
    assert_eq!(ctx1.matches("\r\n").count(), 1);
}
