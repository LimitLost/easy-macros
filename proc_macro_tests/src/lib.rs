use anyhow::Context;
use attributes::{get_attributes, has_attributes};
use helpers::{MacroResult, context};
use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

#[proc_macro_derive(
    DeriveTestStruct,
    attributes(lol, lmao, xlold, xdedd, xnoned, xb, bbb, sql)
)]
#[macro_result::macro_result]
pub fn attributes_test_struct(item: TokenStream) -> anyhow::Result<TokenStream> {
    let parsed = helpers::parse_macro_input!(item as syn::ItemStruct);

    let mut result = MacroResult::default();

    if !has_attributes!(parsed, #[lol]) {
        //Show Compiler error
        result.add(quote! {
            compile_error!("#[lol] attribute on struct not found");
        });
    }

    let mut lol_found = false;
    let mut ded_found = false;
    let mut none_found = false;

    for data in get_attributes!(parsed,#[lmao]#[x__unknown__d]).into_iter() {
        match data
            .to_string()
            .replace(|c: char| c.is_whitespace(), "")
            .as_str()
        {
            "lol" => lol_found = true,
            "ded" => ded_found = true,
            "none" => none_found = true,
            i => {
                let f = format!(
                    "(#[lmao]#[x__unknown__d]) Unexpected attribute on struct: {}",
                    i
                );
                //Show Compiler error
                result.add(quote! {
                    compile_error!(#f);
                });
            }
        }
    }

    if !lol_found {
        //Show Compiler error
        result.add(quote! {
            compile_error!("#[lmao]#[xlold] attribute on struct not found");
        });
    }

    if !ded_found {
        //Show Compiler error
        result.add(quote! {
            compile_error!("#[lmao]#[xdedd] attribute on struct not found");
        });
    }

    if !none_found {
        //Show Compiler error
        result.add(quote! {
            compile_error!("#[lmao]#[xnoned] attribute on struct not found");
        });
    }
    let mut special_found = false;
    let mut dollars_found = false;
    let mut eq_found = false;
    let mut lul_found = false;

    for data in get_attributes!(parsed,#[bbb((lol__unknown__X))]).into_iter() {
        match data
            .to_string()
            .replace(|c: char| c.is_whitespace(), "")
            .as_str()
        {
            "special" => special_found = true,
            "$$$" => dollars_found = true,
            "=" => eq_found = true,
            "((lul))" => lul_found = true,
            i => {
                let f = format!("(lol__unknown__X) Unexpected attribute on struct: {}", i);
                //Show Compiler error
                result.add(quote! {
                    compile_error!(#f);
                });
            }
        }
    }

    if !special_found {
        //Show Compiler error
        result.add(quote! {
            compile_error!("#[bbb((lolspecialX))] attribute on struct not found");
        });
    }

    if !dollars_found {
        //Show Compiler error
        result.add(quote! {
            compile_error!("#[bbb((lol$$$X))] attribute on struct not found");
        });
    }

    if !eq_found {
        //Show Compiler error
        result.add(quote! {
            compile_error!("#[bbb((lol=X))] attribute on struct not found");
        });
    }

    if !lul_found {
        //Show Compiler error
        result.add(quote! {
            compile_error!("#[bbb((lol((lul))X))] attribute on struct not found");
        });
    }

    let mut spec_found = false;
    let mut a_5d_found = false;
    let mut sql_eq_found = false;
    let mut a_25_found = false;

    for data in get_attributes!(parsed,#[sql(table = __unknown__)]).into_iter() {
        match data
            .to_string()
            .replace(|c: char| c.is_whitespace(), "")
            .as_str()
        {
            "spec" => spec_found = true,
            "5d" => a_5d_found = true,
            "=" => sql_eq_found = true,
            "25" => a_25_found = true,
            i => {
                let f = format!(
                    "(#[sql(table = __unknown__)]) Unexpected attribute on struct: {}",
                    i
                );
                //Show Compiler error
                result.add(quote! {
                    compile_error!(#f);
                });
            }
        }
    }

    if !spec_found {
        //Show Compiler error
        result.add(quote! {
            compile_error!("#[sql(table = spec)] attribute on struct not found");
        });
    }

    if !a_5d_found {
        result.add(quote! {
            compile_error!("#[sql(table = 5d)] attribute on struct not found");
        });
    }

    if !sql_eq_found {
        result.add(quote! {
            compile_error!("#[sql(table = =)] attribute on struct not found");
        });
    }

    if !a_25_found {
        result.add(quote! {
            compile_error!("#[sql(table = 25)] attribute on struct not found");
        });
    }

    Ok(result.finalize().into())
}

///Expected Struct:
/// struct TestStruct{
///     field: i32,
/// }
#[proc_macro_attribute]
pub fn macro_test_eq(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(item as syn::ItemStruct);

    let mut result = MacroResult::default();

    let not_real_struct: ItemStruct = syn::parse_quote! {
        struct TestStruct{
            field: i32,
        }
    };

    if parsed != not_real_struct {
        //Show Compiler error
        result.add(quote! {
            compile_error!("Structs are not equal");
        });
    }

    result.finalize().into()
}
