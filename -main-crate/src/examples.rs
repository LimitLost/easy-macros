//! Documentation examples for the easy-macros crate
//!
//! This module contains all the examples used in documentation,
//! marked with #[docify::export] for embedding in docs and README.

#![allow(unused_variables, unused_imports, dead_code)]

use crate::*;

// Feature 1: Automatic Error Context
#[cfg(feature = "always-context")]
mod always_context_examples {
    use super::*;
    use anyhow::{Context, Result};

    struct Data;
    #[derive(Debug)]
    struct User;
    struct Profile {
        id: u64,
    }

    fn find_user(_id: u64) -> Result<User> {
        Ok(User)
    }
    fn load_profile(_user: &User) -> Result<Profile> {
        Ok(Profile { id: 1 })
    }
    fn fetch_data(_id: u64) -> Result<Data> {
        Ok(Data)
    }

    #[docify::export_content]
    #[test]
    #[always_context]
    fn always_context_example() -> Result<()> {
        let user_id = 123u64;
        let user = find_user(user_id)?; // Auto-context with user_id
        let profile = load_profile(&user)?; // Auto-context
        let data = fetch_data(profile.id)?; // Auto-context
        Ok(())
    }
}

// Feature 2: Attribute Pattern Matching
#[cfg(feature = "attributes")]
mod attributes_examples {
    use super::*;
    use anyhow::Context;
    use proc_macro2::TokenStream;
    use syn::{DeriveInput, Field, parse_quote};

    #[docify::export_content]
    #[test]
    fn attributes_comprehensive_example() -> anyhow::Result<()> {
        let input: syn::ItemStruct = parse_quote! {
            #[derive(Debug)]
            #[api_version(v2)]
            struct ApiRoutes {
                #[route(GET, "/users")]
                #[deprecated]
                list_users: String,

                #[route(POST, "/users")]
                create_user: String,

                #[route(GET, "/users/{id}")]
                get_user: String,

                unrouted_field: String,
            }
        };

        // 1. has_attributes! - Check if struct has required attributes
        let is_valid_api = has_attributes!(input, #[derive(Debug)] #[api_version(v2)]);
        assert!(is_valid_api);

        // 2. get_attributes! - Extract API version from struct
        let versions: Vec<TokenStream> = get_attributes!(input, #[api_version(__unknown__)]);
        assert_eq!(versions[0].to_string(), "v2");

        // 3. fields_with_attributes! - Filter only deprecated route fields
        let deprecated: Vec<(usize, &Field)> =
            fields_with_attributes!(&input, #[route(GET, "/users")] #[deprecated]).collect();
        assert_eq!(deprecated.len(), 1);

        // 4. fields_get_attributes! - Extract HTTP methods from all routed fields
        let routes: Vec<(usize, &Field, Vec<TokenStream>)> =
            fields_get_attributes!(&input, #[route(__unknown__, "/users")]);
        assert_eq!(routes.len(), 2); // list_users and create_user
        assert_eq!(routes[0].2[0].to_string(), "GET");
        assert_eq!(routes[1].2[0].to_string(), "POST");

        Ok(())
    }
}

// Feature 3: Exhaustive AST Traversal
#[cfg(feature = "all-syntax-cases")]
mod ast_traversal_examples {
    use super::*;
    use quote::ToTokens;

    struct Context;

    #[docify::export_content]
    #[test]
    fn ast_traversal_example() {
        all_syntax_cases! {
            setup => {
                generated_fn_prefix: "process",
                additional_input_type: &mut Context,
            }
            default_cases => {
                // Called for matching types across entire AST
                fn handle_expr(_expr: &mut syn::Expr, _ctx: &mut Context);

                #[after_system]  // Run after children processed
                fn finalize(_item: &mut syn::Item, _ctx: &mut Context);

                // Handle multiple syn types together (e.g., attributes + generics)
                fn check_attrs_and_generics(
                    _attrs: &mut Vec<syn::Attribute>,
                    _generics: &mut syn::Generics,
                    _ctx: &mut Context
                );
            }
            special_cases => {
                // Override for specific variants
                fn handle_call(_call: &mut syn::ExprCall, _ctx: &mut Context);
            }
        }

        // Function implementations for the handlers
        fn handle_expr(_expr: &mut syn::Expr, _ctx: &mut Context) {
            // Process expressions
        }

        fn finalize(_item: &mut syn::Item, _ctx: &mut Context) {
            // Finalize items after processing children
        }

        fn check_attrs_and_generics(
            _attrs: &mut Vec<syn::Attribute>,
            _generics: &mut syn::Generics,
            _ctx: &mut Context,
        ) {
            // Check attributes and generics together
        }

        fn handle_call(_call: &mut syn::ExprCall, _ctx: &mut Context) {
            // Handle function calls specially
        }
    }
}

// Feature 4: Helper Utilities
#[cfg(feature = "context")]
mod helper_context {
    use super::*;
    use anyhow::Context;

    #[docify::export_content]
    #[test]
    fn helper_context_example() {
        use std::fs;

        fn load_config() -> anyhow::Result<String> {
            // Manual error context with file/line info
            fs::read_to_string("nonexistent.txt").with_context(context!("Loading config"))
        }

        let result = load_config();
        assert!(result.is_err());
    }
}

#[cfg(feature = "tokens-builder")]
mod helper_tokens_builder {
    use super::*;
    use quote::quote;

    #[docify::export_content]
    #[test]
    fn helper_tokens_builder_example() {
        // Token stream builder
        let mut tokens = TokensBuilder::default();
        tokens.add(quote! { println!("Hello"); });
        tokens.braced(); // Wrap in { }

        let stream = tokens.finalize();
        assert!(!stream.is_empty());
    }
}

#[cfg(feature = "indexed-name")]
mod helper_indexed_name {
    use super::*;
    use syn::parse_quote;

    #[docify::export_content]
    #[test]
    fn helper_indexed_name_example() {
        // Generate indexed names: field0, field1, field2
        let names = indexed_name(parse_quote!(field), 3);
        assert_eq!(names.len(), 3);
    }
}

#[cfg(feature = "find-crate")]
mod helper_find_crate {
    use super::*;
    use quote::quote;

    #[docify::export_content]
    #[test]
    fn helper_find_crate_example() {
        // Find crates (handles renames)
        if let Some(path) = find_crate("quote", quote!()) {
            // Use path - quote should be available since we're using it
            assert!(!path.to_string().is_empty());
        }

        let async_rt = find_crate_list(&[
            ("tokio", quote!()),
            ("async-std", quote!()),
            ("nonexistent-crate-xyz", quote!()),
        ]);
        // Should find one of the async runtimes or none
    }
}

// Feature 4: Helper Utilities - Comprehensive Example
#[cfg(all(
    feature = "context",
    feature = "tokens-builder",
    feature = "parse-macro-input",
    feature = "indexed-name",
    feature = "find-crate"
))]
mod helper_utilities_combined {
    use super::*;
    use anyhow::Context;
    use quote::quote;
    use syn::parse_quote;

    #[docify::export_content]
    #[test]
    fn helper_utilities_comprehensive_example() {
        use std::fs;

        // Manual error context with file/line info
        // Feature: `context` (included in `general` and `full`)
        fn load_config() -> anyhow::Result<String> {
            fs::read_to_string("file.txt").with_context(context!("Loading config"))
        }

        // Token stream builder
        // Feature: `tokens-builder` (included in `full`)
        let mut tokens = TokensBuilder::default();
        tokens.add(quote! { println!("Hello"); });
        tokens.braced(); // Wrap in { }
        let stream = tokens.finalize();
        assert!(!stream.is_empty());

        // Generate indexed names: field0, field1, field2
        // Feature: `indexed-name` (included in `full`)
        let names = indexed_name(syn::parse_quote!(field), 3);
        assert_eq!(names.len(), 3);

        // Find crates (handles renames)
        // Feature: `find-crate` (included in `full`)
        if let Some(path) = find_crate("quote", quote!()) {
            assert!(!path.to_string().is_empty());
        }
        // Returns first found crate or None
        let async_rt = find_crate_list(&[("tokio", quote!()), ("async-std", quote!())]);
    }
}

// Complete Example - showing how multiple features work together
#[cfg(all(
    feature = "full",
    feature = "always-context",
    feature = "anyhow-result"
))]
mod complete_example {
    use super::*;
    use anyhow::Context;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{Field, parse_quote};

    #[docify::export_content]
    #[test]
    fn complete_example() -> anyhow::Result<()> {
        // Simulate a proc macro that processes route attributes
        let input: syn::ItemStruct = parse_quote! {
            struct Routes {
                #[route(GET)]
                users: String,
                #[route(POST)]
                create_user: String,
            }
        };

        let routes: Vec<(usize, Field, Vec<TokenStream>)> =
            fields_get_attributes!(input, #[route(__unknown__)]);

        anyhow::ensure!(!routes.is_empty(), "No routes found");

        let mut output = TokensBuilder::default();
        for (_, field, methods) in routes {
            let name = field.ident.as_ref().context("Field needs name")?;
            for method in methods {
                output.add(quote! {
                    pub fn #name() -> Route { Route::new(stringify!(#method)) }
                });
            }
        }

        let result = output.finalize().to_string();
        assert!(result.contains("users"));
        assert!(result.contains("create_user"));
        Ok(())
    }

    struct Route;
    impl Route {
        fn new(_s: &str) -> Self {
            Route
        }
    }
}
