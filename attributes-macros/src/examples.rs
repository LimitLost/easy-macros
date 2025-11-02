//! Documentation examples for the attributes-macros crate
//!
//! This module contains all the examples used in documentation,
//! marked with #[docify::export_content] for embedding in docs and README.

#![allow(unused_variables, unused_imports, dead_code)]

use anyhow::Context;
use attributes::{get_attributes, has_attributes};

// ============================================================================
// Error Handling Examples
// ============================================================================

#[docify::export_content]
#[test] 
fn error_handling_no_matches_example() -> Result<(), Box<dyn std::error::Error>> {
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        #[derive(Debug)]
        struct User;
    };
    
    // No #[route(...)] attributes exist, so this returns empty
    let no_routes: Vec<proc_macro2::TokenStream> = get_attributes!(
        input,
        #[route(__unknown__)]
    );
    assert_eq!(no_routes.len(), 0);
    Ok(())
}

#[docify::export_content]
#[test]
fn error_handling_conditional_missing_example() -> Result<(), Box<dyn std::error::Error>> {
    use syn::parse_quote;

    // Missing the required #[derive(Debug)] attribute
    let input_no_debug: syn::ItemStruct = parse_quote! {
        #[api_version(v1)]
        struct User;
    };
    
    // Without derive(Debug), this returns empty vec![]
    let no_versions: Vec<proc_macro2::TokenStream> = get_attributes!(
        input_no_debug,
        #[derive(Debug)] #[api_version(__unknown__)]
    );
    assert_eq!(no_versions.len(), 0); // Empty because derive(Debug) is missing
    Ok(())
}

// ============================================================================
// has_attributes! examples
// ============================================================================

#[docify::export_content]
#[test]
fn has_attributes_basic_usage() {
    use syn::parse_quote;

    // Check for a single attribute
    let input: syn::ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[serde(rename_all = "camelCase")]
        struct User {
            name: String,
        }
    };

    let has_debug = has_attributes!(input, #[derive(Debug)]);
    assert!(has_debug);

    // Check for multiple attributes (all must be present)
    let has_both = has_attributes!(
        input,
        #[derive(Debug)] #[serde(rename_all = "camelCase")]
    );
    assert!(has_both);

    // This returns false since #[derive(Clone)] is not present
    let has_clone = has_attributes!(input, #[derive(Clone)]);
    assert!(!has_clone);
}

#[docify::export_content]
#[test]
fn has_attributes_field_attributes() {
    use syn::parse_quote;

    // Works with any item that has attributes
    let field: syn::Field = parse_quote! {
        #[serde(skip)]
        #[validate]
        pub name: String
    };

    let field_has_serde = has_attributes!(field, #[serde(skip)]);
    assert!(field_has_serde);
}

#[docify::export_content]
#[test]
fn has_attributes_exact_matching() {
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        struct Foo;
    };

    // This returns FALSE because it's looking for ONLY Debug
    let only_debug = has_attributes!(input, #[derive(Debug)]);
    assert!(!only_debug);

    // Must match exactly
    let debug_clone = has_attributes!(input, #[derive(Debug, Clone)]);
    assert!(debug_clone);
}

// ============================================================================
// get_attributes! examples
// ============================================================================

#[docify::export_content]
#[test]
fn get_attributes_basic_value_extraction() -> Result<(), Box<dyn std::error::Error>> {
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        #[serde(rename = "custom_name")]
        #[serde(rename = "other_name")]
        struct User {
            name: String,
        }
    };

    // Extract rename values
    let renames: Vec<proc_macro2::TokenStream> = get_attributes!(
        input,
        #[serde(rename = __unknown__)]
    );

    assert_eq!(renames.len(), 2);
    assert_eq!(renames[0].to_string(), "\"custom_name\"");
    assert_eq!(renames[1].to_string(), "\"other_name\"");
    Ok(())
}

#[docify::export_content]
#[test]
fn get_attributes_partial_identifier_matching() -> Result<(), Box<dyn std::error::Error>> {
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        #[test_case_one]
        #[test_case_two]
        #[test_case_foo]
        #[other_attr]
        struct TestSuite;
    };

    // Extract the suffix after "test_case_"
    let test_cases: Vec<proc_macro2::TokenStream> = get_attributes!(
        input,
        #[test_case___unknown__]
    );

    assert_eq!(test_cases.len(), 3);
    assert_eq!(test_cases[0].to_string(), "one");
    assert_eq!(test_cases[1].to_string(), "two");
    assert_eq!(test_cases[2].to_string(), "foo");
    Ok(())
}

#[docify::export_content]
#[test]
fn get_attributes_function_parameter_extraction() -> Result<(), Box<dyn std::error::Error>> {
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        #[route(GET, "/users")]
        #[route(POST, "/users/{id}")]
        #[route(DELETE, "/admin/users")]
        struct ApiRoutes;
    };

    // Extract HTTP methods
    let methods: Vec<proc_macro2::TokenStream> = get_attributes!(
        input,
        #[route(__unknown__, "/users")]
    );
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].to_string(), "GET");

    // Extract all paths
    let paths: Vec<proc_macro2::TokenStream> = get_attributes!(
        input,
        #[route(POST, __unknown__)]
    );
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].to_string(), "\"/users/{id}\"");
    Ok(())
}

#[docify::export_content]
#[test]
fn get_attributes_nested_example() -> Result<(), Box<dyn std::error::Error>> {
    use syn::parse_quote;

    // Docify has trouble parsing parse_quote! with attributes inside,
    // so we use a string literal workaround
    let input: syn::ItemStruct = syn::parse_str(
        r#"
        #[config(database(url = "postgres://localhost"))]
        #[config(redis(url = "redis://localhost"))]
        struct AppConfig;
    "#,
    )?;

    // Extract database URL
    let db_urls: Vec<proc_macro2::TokenStream> = get_attributes!(
        input,
        #[config(database(url = __unknown__))]
    );
    assert_eq!(db_urls[0].to_string(), "\"postgres://localhost\"");
    Ok(())
}

#[docify::export_content]
#[test]
fn get_attributes_conditional_extraction() -> Result<(), Box<dyn std::error::Error>> {
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[api_version(v1)]
        #[api_version(v2)]
        struct User;
    };

    // Only extract if derive(Debug) is also present
    let versions: Vec<proc_macro2::TokenStream> = get_attributes!(
        input,
        #[derive(Debug)] #[api_version(__unknown__)]
    );

    // If the item has BOTH #[derive(Debug)] AND #[api_version(...)],
    // this returns the extracted values from api_version attributes.
    // If derive(Debug) is missing, returns empty vec![]
    assert_eq!(versions.len(), 2); // Only if derive(Debug) exists
    assert_eq!(versions[0].to_string(), "v1");
    assert_eq!(versions[1].to_string(), "v2");

    // Without derive(Debug), this would return vec![]
    let input_no_debug: syn::ItemStruct = parse_quote! {
        #[api_version(v1)]
        struct User;
    };
    let no_versions: Vec<proc_macro2::TokenStream> = get_attributes!(
        input_no_debug,
        #[derive(Debug)] #[api_version(__unknown__)]
    );
    assert_eq!(no_versions.len(), 0); // Empty because derive(Debug) is missing
    Ok(())
}

#[docify::export_content]
#[test]
fn get_attributes_exact_matching_required() -> Result<(), Box<dyn std::error::Error>> {
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        #[route(GET, "/users", auth = true)]
        struct Handler;
    };

    // This matches the exact attribute structure
    let methods = get_attributes!(input, #[route(__unknown__, "/users", auth = true)]);
    assert_eq!(methods.len(), 1);

    // This does NOT match (missing auth = true)
    let no_match = get_attributes!(input, #[route(__unknown__, "/users")]);
    assert_eq!(no_match.len(), 0);
    Ok(())
}

// ============================================================================
// fields_with_attributes! examples
// ============================================================================

#[docify::export_content]
#[test]
fn fields_with_attributes_basic_filtering() {
    use attributes::fields_with_attributes;
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        struct User {
            #[serde(skip)]
            id: u64,

            #[validate]
            #[serde(rename = "user_name")]
            name: String,

            #[validate]
            email: String,

            created_at: String,
        }
    };

    // Get fields with validation attributes
    let validated_fields: Vec<(usize, syn::Field)> = fields_with_attributes!(
        input,
        #[validate]
    )
    .collect();

    assert_eq!(validated_fields.len(), 2); // name and email fields
    assert_eq!(validated_fields[0].0, 1); // name is at index 1
    assert_eq!(validated_fields[1].0, 2); // email is at index 2
}

#[docify::export_content]
#[test]
fn fields_with_attributes_multiple_requirements() {
    use attributes::fields_with_attributes;
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        struct User {
            #[serde(skip)]
            id: u64,

            #[validate]
            #[serde(rename = "user_name")]
            name: String,

            #[validate]
            email: String,
        }
    };

    // Get fields that have BOTH validate AND the exact serde attribute
    let validated_serde_fields: Vec<(usize, syn::Field)> = fields_with_attributes!(
        input,
        #[validate] #[serde(rename = "user_name")]
    )
    .collect();

    assert_eq!(validated_serde_fields.len(), 1); // only name field has both
}

#[docify::export_content]
#[test]
fn fields_with_attributes_borrowing() {
    use attributes::{fields_with_attributes, fields_with_attributes_debug};
    use syn::parse_quote;

    let mut input: syn::ItemStruct = parse_quote! {
        struct Config {
            #[required]
            database_url: String,

            #[optional]
            redis_url: Option<String>,
        }
    };

    // Use immutable reference to avoid consuming input
    let required_fields: Vec<(usize, &syn::Field)> = fields_with_attributes!(
        &input,
        #[required]
    )
    .collect();

    // input is still available for use
    assert_eq!(required_fields.len(), 1);

    // Use mutable reference to potentially modify fields
    let mutable_fields: Vec<(usize, &mut syn::Field)> = fields_with_attributes!(
        &mut input,
        #[required]
    )
    .collect();

    // Can now modify the fields if needed
    assert_eq!(mutable_fields.len(), 1);
}

// ============================================================================
// fields_get_attributes! examples
// ============================================================================

#[docify::export_content]
#[test]
fn fields_get_attributes_route_extraction() -> Result<(), Box<dyn std::error::Error>> {
    use attributes::{fields_get_attributes, fields_get_attributes_debug};
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        struct ApiEndpoints {
            #[route(GET, "/users")]
            get_users: String,

            #[route(POST, "/users")]
            create_user: String,

            #[route(GET, "/users/{id}")]
            get_user: String,

            #[route(DELETE, "/users/{id}")]
            delete_user: String,

            #[other_attr]
            non_route_field: String,
        }
    };

    // Extract HTTP methods for all route fields
    let methods: Vec<(usize, syn::Field, Vec<proc_macro2::TokenStream>)> =
        fields_get_attributes!(input, #[route(__unknown__, "/users")]);

    assert_eq!(methods.len(), 2); // get_users and create_user
    assert_eq!(methods[0].2[0].to_string(), "GET"); // get_users method
    assert_eq!(methods[1].2[0].to_string(), "POST"); // create_user method

    Ok(())
}

#[docify::export_content]
#[test]
fn fields_get_attributes_database_columns() -> Result<(), Box<dyn std::error::Error>> {
    use attributes::fields_get_attributes;
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        struct UserTable {
            #[column(id, primary_key)]
            #[column(id, auto_increment)]
            id: i32,

            #[column(varchar, length = 255)]
            #[unique]
            email: String,

            #[column(varchar, length = 100)]
            #[nullable]
            name: Option<String>,

            #[column(timestamp)]
            created_at: String,
        }
    };

    // Extract column types
    let column_types: Vec<(usize, syn::Field, Vec<proc_macro2::TokenStream>)> =
        fields_get_attributes!(input, #[column(__unknown__, length = 255)]);

    assert_eq!(column_types.len(), 1); // only email field
    assert_eq!(column_types[0].2[0].to_string(), "varchar");

    Ok(())
}

#[docify::export_content]
#[test]
fn fields_get_attributes_validation_rules() -> Result<(), Box<dyn std::error::Error>> {
    use attributes::fields_get_attributes;
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        struct UserForm {
            #[validate(length(min = 3, max = 50))]
            #[validate(regex = r"^[a-zA-Z\s]+$")]
            name: String,

            #[validate(email)]
            #[validate(length(max = 100))]
            email: String,

            #[validate(range(min = 18, max = 120))]
            age: u8,

            #[no_validation]
            created_at: String,
        }
    };

    // Extract minimum length requirements
    let min_lengths: Vec<(usize, syn::Field, Vec<proc_macro2::TokenStream>)> =
        fields_get_attributes!(input, #[validate(length(min = __unknown__, max = 50))]);

    assert_eq!(min_lengths.len(), 1); // only name field
    assert_eq!(min_lengths[0].2[0].to_string(), "3");

    Ok(())
}

#[docify::export_content]
#[test]
fn fields_get_attributes_multiple_matches_per_field() -> Result<(), Box<dyn std::error::Error>> {
    use attributes::fields_get_attributes;
    use syn::parse_quote;

    // **KEY CONCEPT**: A single field can have MULTIPLE matching attributes!
    // Each matching attribute on the same field adds to the Vec<TokenStream>
    let input: syn::ItemStruct = parse_quote! {
        struct Multi {
            #[tag(v1)]  // ← All three of these match #[tag(__unknown__)]
            #[tag(v2)]  // ←
            #[tag(v3)]  // ←
            field: String,
        }
    };

    let versions: Vec<(usize, syn::Field, Vec<proc_macro2::TokenStream>)> =
        fields_get_attributes!(input, #[tag(__unknown__)]);

    assert_eq!(versions.len(), 1); // ONE field in results
    assert_eq!(versions[0].2.len(), 3); // THREE extracted values from that field
    assert_eq!(versions[0].2[0].to_string(), "v1");
    assert_eq!(versions[0].2[1].to_string(), "v2");
    assert_eq!(versions[0].2[2].to_string(), "v3");

    Ok(())
}

#[docify::export_content]
#[test]
fn fields_get_attributes_borrowing() -> Result<(), Box<dyn std::error::Error>> {
    use attributes::{fields_get_attributes, fields_get_attributes_debug};
    use syn::parse_quote;

    let mut large_struct: syn::ItemStruct = parse_quote! {
        struct LargeModel {
            #[indexed(btree)]
            id: i32,
        }
    };

    // Use references to avoid moving large struct
    let indexed_fields: Vec<(usize, &syn::Field, Vec<proc_macro2::TokenStream>)> =
        fields_get_attributes!(&large_struct, #[indexed(__unknown__)]);

    // large_struct is still available
    assert_eq!(indexed_fields.len(), 1);

    // Use mutable references if you need to modify fields
    let indexed_mut: Vec<(usize, &mut syn::Field, Vec<proc_macro2::TokenStream>)> =
        fields_get_attributes!(&mut large_struct, #[indexed(__unknown__)]);

    for (_idx, field, index_types) in indexed_mut {
        // Can modify field attributes here
        assert_eq!(index_types[0].to_string(), "btree");
    }

    Ok(())
}

#[docify::export_content]
#[test]
fn fields_get_attributes_complex_pattern() -> Result<(), Box<dyn std::error::Error>> {
    use attributes::fields_get_attributes;
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        struct ApiModel {
            #[serialize(json, format = "iso8601")]
            #[serialize(xml, format = "rfc3339")]
            timestamp: String,

            #[serialize(json, omit_null = true)]
            optional_field: Option<String>,
        }
    };

    // Extract JSON serialization formats
    let json_formats: Vec<(usize, syn::Field, Vec<proc_macro2::TokenStream>)> =
        fields_get_attributes!(input, #[serialize(json, format = __unknown__)]);

    assert_eq!(json_formats.len(), 1);
    assert_eq!(json_formats[0].2[0].to_string(), "\"iso8601\"");

    Ok(())
}
