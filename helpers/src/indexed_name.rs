pub fn indexed_name(name: syn::Ident, count: usize) -> Vec<syn::Ident> {
    let mut names = Vec::new();
    for i in 0..count {
        let indexed_name = quote::format_ident!("{}{}", name, i);
        names.push(indexed_name);
    }
    names
}
