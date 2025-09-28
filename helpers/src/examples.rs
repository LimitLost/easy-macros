//! Documentation examples for the easy-macros-helpers-macro-safe crate
//!
//! This module contains all the examples used in documentation,
//! marked with #[docify::export] for embedding in docs and README.

#![allow(unused_variables, unused_imports, dead_code)]

use crate::*;
use proc_macro2::TokenStream;
use quote::quote;
use std::error::Error;
use syn::parse_quote;

// Context macro examples
#[cfg(feature = "context")]
mod context_examples {
    use super::*;
    use anyhow::Context;

    #[docify::export_content]
    #[test]
    fn readme_context_basic_example() {
        use std::fs;

        fn load_config() -> anyhow::Result<String> {
            // Context with a custom message - this will be the failing operation
            fs::read_to_string("settings.json")
                .with_context(context!("Failed to load application settings"))
        }

        let result = load_config();
        assert!(result.is_err());

        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Failed to load application settings"));
        assert!(error_msg.contains("examples.rs"));
    }
}

// TokensBuilder examples
#[cfg(feature = "full")]
mod full_examples {
    use super::*;

    #[docify::export_content]
    #[test]
    fn tokens_builder_basic_usage() {
        let mut result = TokensBuilder::default();

        // Add multiple token streams
        result.add(quote! { let x = 1; });
        result.add(quote! { let y = 2; });
        result.add(quote! { println!("{} + {} = {}", x, y, x + y); });

        // Wrap in braces to create a block
        result.braced();

        let tokens = result.finalize();
        assert_eq!(
            readable_token_stream(&tokens.to_string()),
            "{ let x = 1; let y = 2; println!(\"{} + {} = {}\", x, y, x + y); }"
        );
    }

    #[docify::export_content]
    fn tokens_builder_add_example() {
        let mut result = TokensBuilder::default();
        result.add(quote! { fn hello() });
        result.add(quote! { { println!("Hello, world!"); } });

        let tokens = result.finalize();
        assert_eq!(
            readable_token_stream(&tokens.to_string()),
            "fn hello() { println!(\"Hello, world!\"); }"
        );
    }

    #[docify::export_content]
    #[test]
    fn tokens_builder_braced_example() {
        let mut result = TokensBuilder::default();
        result.add(quote! { let x = 42; });
        result.add(quote! { x * 2 });
        result.braced();

        let tokens = result.finalize();
        assert_eq!(
            readable_token_stream(&tokens.to_string()),
            "{ let x = 42; x * 2 }"
        );
    }

    #[docify::export_content]
    #[test]
    fn tokens_builder_finalize_example() {
        let mut result = TokensBuilder::default();
        result.add(quote! { println!("Done!"); });

        let final_tokens = result.finalize();
        assert_eq!(
            readable_token_stream(&final_tokens.to_string()),
            "println!(\"Done!\");"
        );
    }

    // README TokensBuilder example - using extern crate name for external users
    #[docify::export_content]
    #[test]
    fn readme_tokens_builder_example() {
        let mut result = TokensBuilder::default();

        // Add multiple token streams
        result.add(quote! {
            println!("Hello");
        });
        result.add(quote! {
            println!("World");
        });

        // Wrap everything in braces
        result.braced();

        // Get final result
        let tokens = result.finalize();
        assert_eq!(
            readable_token_stream(&tokens.to_string()),
            "{ println!(\"Hello\"); println!(\"World\"); }"
        );
    }

    // indexed_name examples

    #[docify::export_content]
    #[test]
    fn indexed_name_basic_example() {
        let base = syn::parse_quote!(field);
        let names = indexed_name(base, 3);

        // Use in a quote! macro to generate struct fields
        let output = quote! {
            struct MyStruct {
                #(#names: i32,)*
            }
        };
        assert_eq!(
            readable_token_stream(&output.to_string()),
            "struct MyStruct { field0: i32, field1: i32, field2: i32, }"
        );
    }

    #[docify::export_content]
    #[test]
    fn readme_indexed_name_example() {
        let field_names = indexed_name(syn::parse_quote!(field), 3);
        let output = quote! {
            struct MyStruct {
                #(#field_names: i32,)*
            }
        };
        assert_eq!(
            readable_token_stream(&output.to_string()),
            "struct MyStruct { field0: i32, field1: i32, field2: i32, }"
        );
    }

    // find_crate examples

    #[docify::export_content]
    fn find_crate_basic_usage() {
        // Find a crate without additional path
        if let Some(path) = find_crate("serde", quote!()) {
            // Returns: `serde` or `crate` depending on context
        }

        // Find a crate with additional path segments
        if let Some(path) = find_crate("my_crate", quote!(::utils::helper)) {
            // Returns: `my_crate::utils::helper` or `crate::utils::helper`
        }

        // With a renamed crate in Cargo.toml:
        // [dependencies]
        // serde_renamed = { package = "serde", version = "1.0" }
        if let Some(path) = find_crate("serde", quote!(::Serialize)) {
            // Returns: `serde_renamed::Serialize`
        }
    }

    #[docify::export_content]
    fn find_crate_list_basic_example() {
        let crates = &[
            ("tokio", quote!(::runtime)),
            ("async-std", quote!(::task)),
            ("smol", quote!()),
        ];

        if let Some(async_runtime) = find_crate_list(crates) {
            // Uses the first available async runtime crate
        }
    }

    #[docify::export_content]
    fn find_crate_list_renamed_example() {
        // With renamed crates in Cargo.toml:
        // [dependencies]
        // tokio_renamed = { package = "tokio", version = "1.0" }
        // async_std = "1.0"

        // `tokio` - Will find "tokio_renamed"
        // `async-std` - Will find "async_std"
        let crates = &[("tokio", quote!(::runtime)), ("async-std", quote!(::task))];
        if let Some(path) = find_crate_list(crates) {
            // Returns: tokio_renamed::runtime
        }
    }

    #[docify::export_content]
    fn readme_find_crate_example() {
        // Simple crate lookup
        if let Some(path) = find_crate("serde", quote!(::Serialize)) {
            // Use path in generated code
        }

        // Handles renamed crates automatically
        // If Cargo.toml has: serde_renamed = { package = "serde", version = "1.0" }
        // The above will return: serde_renamed::Serialize

        // Try multiple crates with fallbacks
        let async_crates = &[
            ("tokio", quote!(::runtime::Runtime)),
            ("async-std", quote!(::task)),
            ("smol", quote!()),
        ];

        if let Some(async_path) = find_crate_list(async_crates) {
            // Uses first available async runtime
        }
    }
    // CompileErrorProvider examples

    #[docify::export_content]
    #[test]
    fn error_data_basic_usage() {
        let mut errors = Vec::<String>::new();
        errors.push("Invalid syntax".to_string());
        errors.push("Missing required field".to_string());

        assert!(!errors.no_errors());
        let error_messages = errors.error_data();
        assert!(errors.no_errors());
    }

    #[docify::export_content]
    #[test]
    fn error_data_custom_implementation() {
        #[derive(Default)]
        struct ValidationErrors {
            errors: Vec<String>,
            other_data: String,
        }

        impl ValidationErrors {
            fn add_error(&mut self, msg: &str) {
                self.errors.push(msg.to_string());
            }
        }

        impl CompileErrorProvider for ValidationErrors {
            fn no_errors(&self) -> bool {
                self.errors.no_errors()
            }

            fn error_data(&mut self) -> Vec<String> {
                self.errors.error_data()
            }
        }

        let mut validator = ValidationErrors::default();
        validator.add_error("Field 'name' is required");
        validator.add_error("Field 'age' must be a positive number");

        assert!(!validator.no_errors());
        let messages = validator.error_data();
        assert_eq!(messages.len(), 2);
        assert!(validator.no_errors());
    }

    // expr_error_wrap examples

    #[docify::export_content]
    #[test]
    fn expr_error_wrap_basic_usage() {
        let mut expr = parse_quote!(42);
        let mut errors = vec![
            "This is a warning".to_string(),
            "Another issue found".to_string(),
        ];

        expr_error_wrap(&mut expr, &mut errors);

        assert_eq!(
            quote! { #expr }.to_string(),
            quote! {
                {
                    compile_error!("This is a warning");
                    compile_error!("Another issue found");
                    42
                }
            }
            .to_string()
        );
    }

    #[docify::export_content]
    #[test]
    fn expr_error_wrap_custom_validator() {
        #[derive(Default)]
        struct MacroValidator {
            errors: Vec<String>,
            context: String,
        }

        impl MacroValidator {
            fn new(context: &str) -> Self {
                Self {
                    errors: Vec::new(),
                    context: context.to_string(),
                }
            }

            fn validate_field(&mut self, field_name: &str, is_valid: bool) {
                if !is_valid {
                    self.errors.push(format!(
                        "Invalid field '{}' in {}",
                        field_name, self.context
                    ));
                }
            }
        }

        impl CompileErrorProvider for MacroValidator {
            fn no_errors(&self) -> bool {
                self.errors.no_errors()
            }

            fn error_data(&mut self) -> Vec<String> {
                self.errors.error_data()
            }
        }

        let mut validator = MacroValidator::new("MyStruct");
        validator.validate_field("name", false);
        validator.validate_field("id", false);

        let mut result_expr = parse_quote!(MyStruct::default());
        expr_error_wrap(&mut result_expr, &mut validator);

        assert_eq!(
            quote! { #result_expr }.to_string(),
            quote! {
                {
                    compile_error!("Invalid field 'name' in MyStruct");
                    compile_error!("Invalid field 'id' in MyStruct");
                    MyStruct::default()
                }
            }
            .to_string()
        );
    }

    #[docify::export_content]
    #[test]
    fn readme_error_wrapping_example() {
        let mut errors = Vec::<String>::new();
        let mut expr = syn::parse_quote!(some_expression);

        // Add some errors
        errors.push("This field is required".to_string());
        errors.push("Invalid type specified".to_string());

        // Wrap expression with compile errors
        expr_error_wrap(&mut expr, &mut errors);
        assert_eq!(
            quote! { #expr }.to_string(),
            quote! {
                {
                    compile_error!("This field is required");
                    compile_error!("Invalid type specified");
                    some_expression
                }
            }
            .to_string()
        );
    }

    // token_stream_to_consistent_string example

    #[docify::export_content]
    #[test]
    fn token_stream_consistent_string_example() {
        let tokens = quote! { fn hello() -> String { "hello world".to_string() } };
        let result = token_stream_to_consistent_string(tokens);
        assert_eq!(result, "fnhello()->String{\"hello world\".to_string()}");
    }

    #[docify::export_content]
    #[test]
    fn readable_token_stream_example() {
        let spaced = "Vec < String >";
        let clean = readable_token_stream(spaced);
        assert_eq!(clean, "Vec<String>");

        let input = "a  b   c";
        let clean = readable_token_stream(input);
        assert_eq!(clean, "a b c");
    }
}
