/// Generates a vector of identifiers by appending numeric indices to a base name.
///
/// This function is useful in procedural macros when you need to generate multiple
/// similar identifiers, such as field names, variable names, or function parameters.
///
/// # Arguments
///
/// * `name` - The base identifier to which indices will be appended
/// * `count` - The number of indexed identifiers to generate (0 to count-1)
///
/// # Returns
///
/// A vector of `syn::Ident` with numeric suffixes: `[name0, name1, name2, ...]`
///
/// # Examples
///
#[doc = docify::embed!("src/examples.rs", indexed_name_basic_example)]
///
/// # Use Cases
///
/// - Creating multiple similar variables in generated code
/// - Building function parameter lists with indexed names
pub fn indexed_name(name: syn::Ident, count: usize) -> Vec<syn::Ident> {
    let mut names = Vec::new();
    for i in 0..count {
        let indexed_name = quote::format_ident!("{}{}", name, i);
        names.push(indexed_name);
    }
    names
}
