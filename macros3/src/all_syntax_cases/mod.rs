//Example with input
/*
// Only Token![] calls are supported, calling by specific Token Type is not
all_syntax_cases!{
    setup => {
        entry_ty: syn::Item,
        fn_name_prefix: "example_",
        additional_input_ty: Option<NoContext>,
    }
    default_cases => {
        fn example_default(attrs: &mut Vec<Attribute>, no_context: &mut Option<NoContext>);
    }
    special_cases => {
        //Mutable data request should not be allowed here
        fn example_try(expr_try: syn::ExprTry, no_context: Option<NoContext>) ;
        fn example_macro(expr_macro: syn::ExprMacro, no_context: Option<NoContext>) ;
    }

}
 */

mod data;

use data::{EssentialFnData, MacroData};
use macros4::matched_check;
use proc_macro::TokenStream;
use proc_macro2::{Group, TokenTree};
use quote::{ToTokens, quote};
use syn::{
    Abi, Arm, Attribute, Block, Expr, FieldValue, Fields, FieldsNamed, ForeignItem, Generics,
    Ident, ImplItem, ImplRestriction, Item, Macro, Path, Signature, StaticMutability, Token,
    TraitItem, Type, TypeParamBound, UseTree, Variant, Visibility, punctuated::Punctuated,
};

//TODO Handle items
//TODO Handle expressions
//TODO Handle Generics (some have expr inside of them)

//TODO Create a list of every type found that can be used in default or special case (while computing this macro)
//TODO If passed in default or special function are never called show an error

fn item_search(macro_data: &mut MacroData) -> proc_macro2::TokenStream {
    let MacroData {
        fn_name_prefix,
        additional_input_ty,
        default_functions,
        special_functions,
    } = macro_data;

    let additional_input_name = quote::format_ident!("__additional_input");

    let mut result_matches = proc_macro2::TokenStream::new();

    //Uses `result_matches`, `default_functions` and `special_functions`, without requesting them in macro input
    matched_check!(syn::Item::Const(syn::ItemConst {
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub const_token: Token![const],
        pub ident: Ident,
        pub generics: Generics,
        pub colon_token: Token![:],
        pub ty: Box<Type>,
        pub eq_token: Token![=],
        pub expr: Box<Expr>,
        pub semi_token: Token![;],
    }));

    matched_check!(syn::Item::Enum(syn::ItemEnum{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub enum_token: Token![enum],
        pub ident: Ident,
        pub generics: Generics,
        pub brace_token: syn::token::Brace,
        pub variants: Punctuated<Variant, Token![,]>,
    }));
    matched_check!(syn::Item::ExternCrate(syn::ItemExternCrate{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub extern_token: Token![extern],
        pub crate_token: Token![crate],
        pub ident: Ident,
        pub rename: Option<(Token![as], Ident)>,
        pub semi_token: Token![;],
    }));
    matched_check!(syn::Item::Fn(syn::ItemFn{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub sig: Signature,
        pub block: Box<Block>,
    }));
    matched_check!(syn::Item::ForeignMod(syn::ItemForeignMod{
        pub attrs: Vec<Attribute>,
        pub unsafety: Option<Token![unsafe]>,
        pub abi: Abi,
        pub brace_token: syn::token::Brace,
        pub items: Vec<ForeignItem>,
    }));
    matched_check!(syn::Item::Impl(syn::ItemImpl{
        pub attrs: Vec<Attribute>,
        pub defaultness: Option<Token![default]>,
        pub unsafety: Option<Token![unsafe]>,
        pub impl_token: Token![impl],
        pub generics: Generics,
        pub trait_: Option<(Option<Token![!]>, Path, Token![for])>,
        pub self_ty: Box<Type>,
        pub brace_token: syn::token::Brace,
        pub items: Vec<ImplItem>,
    }));
    matched_check!(syn::Item::Macro(syn::ItemMacro{
        pub attrs: Vec<Attribute>,
        /// The `example` in `macro_rules! example { ... }`.
        pub ident: Option<Ident>,
        pub mac: Macro,
        pub semi_token: Option<Token![;]>,
    }));
    matched_check!(syn::Item::Mod(syn::ItemMod{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub unsafety: Option<Token![unsafe]>,
        pub mod_token: Token![mod],
        pub ident: Ident,
        pub content: Option<(syn::token::Brace, Vec<Item>)>,
        pub semi: Option<Token![;]>,
    }));
    matched_check!(syn::Item::Static(syn::ItemStatic{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub static_token: Token![static],
        pub mutability: StaticMutability,
        pub ident: Ident,
        pub colon_token: Token![:],
        pub ty: Box<Type>,
        pub eq_token: Token![=],
        pub expr: Box<Expr>,
        pub semi_token: Token![;],
    }));
    matched_check!(syn::Item::Struct(syn::ItemStruct{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub struct_token: Token![struct],
        pub ident: Ident,
        pub generics: Generics,
        pub fields: Fields,
        pub semi_token: Option<Token![;]>,
    }));
    matched_check!(syn::Item::Trait(syn::ItemTrait{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub unsafety: Option<Token![unsafe]>,
        pub auto_token: Option<Token![auto]>,
        pub restriction: Option<ImplRestriction>,
        pub trait_token: Token![trait],
        pub ident: Ident,
        pub generics: Generics,
        pub colon_token: Option<Token![:]>,
        pub supertraits: Punctuated<TypeParamBound, Token![+]>,
        pub brace_token: syn::token::Brace,
        pub items: Vec<TraitItem>,
    }));
    matched_check!(syn::Item::TraitAlias(syn::ItemTraitAlias{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub trait_token: Token![trait],
        pub ident: Ident,
        pub generics: Generics,
        pub eq_token: Token![=],
        pub bounds: Punctuated<TypeParamBound, Token![+]>,
        pub semi_token: Token![;],
    }));
    matched_check!(syn::Item::Type(syn::ItemType{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub type_token: Token![type],
        pub ident: Ident,
        pub generics: Generics,
        pub eq_token: Token![=],
        pub ty: Box<Type>,
        pub semi_token: Token![;],
    }));
    matched_check!(syn::Item::Union(syn::ItemUnion{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub union_token: Token![union],
        pub ident: Ident,
        pub generics: Generics,
        pub fields: FieldsNamed,
    }));
    matched_check!(syn::Item::Use(syn::ItemUse{
        pub attrs: Vec<Attribute>,
        pub vis: Visibility,
        pub use_token: Token![use],
        pub leading_colon: Option<Token![::]>,
        pub tree: UseTree,
        pub semi_token: Token![;],
    }));

    result_matches.extend(quote! {
        syn::Item::Verbatim(token_stream) => {
            todo!("syn::Item::Verbatim is unsupported by all_syntax_cases macro")
        }
        i => todo!(
            "Item not supported yet by all_syntax_cases macro | Item: {}",
            i.to_token_stream()
        ),
    });

    let fn_name = quote::format_ident!("{}{}", fn_name_prefix, "item_handle");

    quote! {
        fn
    }
}

///Creates a function covering all cases of provided type
/// additional_input is passed in deeper as a copy, not a mutable reference
/// Every item in for example block has it's own copy of additional_input
pub fn all_syntax_cases(item: TokenStream) -> TokenStream {}
