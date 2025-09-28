#[doc(hidden)]
pub use context_internal::{context_internal, context_internal2};

#[macro_export]
/// Creates a closure that generates context strings for error handling with automatic file and line information.
///
/// This macro provides a convenient way to add context to errors when using the [anyhow](https://crates.io/crates/anyhow)
/// crate's `.with_context()` method. It automatically prepends the current file name and line number to your
/// context message, making error tracking much easier during debugging.
///
/// The macro supports the same syntax as the standard [`format!`] macro, allowing for formatted context messages
/// with placeholders and arguments. When no arguments are provided, it creates a simple context with just
/// file and line information.
///
/// # Syntax
///
/// ```ignore
/// context!()                          // Just file:line info
/// context!("message")                 // Static message with file:line
/// context!("format {}", arg)          // Formatted message with file:line
/// context!("multiple {} {}", a, b)    // Multiple format arguments
/// context!("multiple {a} {b}")        // All things that format! supports are supported here too
/// ```
///
/// # Returns
///
/// Returns a closure of type `impl FnOnce() -> String` that can be passed directly to
/// anyhow's `.with_context()` method or called manually to get the formatted context string.
///
/// # Output Format
///
/// The context macro produces strings in the following exact formats:
///
/// - **With no message:** `"src/file.rs:line_number"`  
///   Example: `"src/main.rs:42"`
///
/// - **With message:** `"src/file.rs:line_number\r\nYour custom message here"`  
///   Example: `"src/main.rs:42\r\nOperation failed"`
///
/// The file path includes the `src/` prefix and the line number is automatically determined
/// at compile time using the [`file!`] and [`line!`] macros. Messages are separated from
/// the location info with a carriage return + line feed (`\r\n`) sequence.
///
/// # Examples
///
/// ## Basic Usage
///
#[doc = docify::embed!("src/examples.rs", context_basic_usage_example)]
///
/// ## With Custom Messages
///
#[doc = docify::embed!("src/examples.rs", context_with_custom_message_example)]
///
/// ## With Formatted Messages
///
#[doc = docify::embed!("src/examples.rs", context_with_formatted_message_example)]
///
/// ## Chaining Multiple Context Levels
///
#[doc = docify::embed!("src/examples.rs", context_chaining_multiple_levels_example)]
///
/// ## Manual Context Generation
///
#[doc = docify::embed!("src/examples.rs", context_manual_generation_example)]
///
/// # See Also
///
/// - [`anyhow::Context`](https://docs.rs/anyhow/latest/anyhow/trait.Context.html) - The trait that provides the `.with_context()` method
/// - [`format!`] - The standard formatting macro that this macro's syntax is based on
/// - [`file!`] and [`line!`] - The macros used internally to get location information
macro_rules! context {
    () => {
        || {
            $crate::context_internal!()
        }
    };
    ($($arg:tt)*) => {
        || {
            // Adds syntax checking from format! macro
            let _ = || {
                let _ = format!($($arg)*);
            };
            $crate::context_internal!($($arg)*)
        }
    };
}
