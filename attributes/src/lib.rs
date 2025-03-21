use std::ops::Range;

use always_context::always_context;
use anyhow::Context;
use helpers_context::context;
use helpers_macro_safe::{MacroResult, indexed_name, parse_macro_input};
use lazy_static::lazy_static;
use macro_result::macro_result;
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::{ToTokens, quote};
use syn::parse::Parse;

lazy_static! {
    static ref UNKNOWN: &'static str = "__unknown__";
}

struct HandleAttributesInput {
    operate_on: syn::Expr,
    _comma: syn::token::Comma,
    attributes: Vec<syn::Attribute>,
}

impl syn::parse::Parse for HandleAttributesInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let operate_on = input.parse()?;
        let _comma = input.parse()?;
        let attributes = syn::Attribute::parse_outer(input)?;

        Ok(HandleAttributesInput {
            operate_on,
            _comma,
            attributes,
        })
    }
}

#[proc_macro]
#[macro_result]
///Returns true if the passed in item has all passed in attributes (one or more)
pub fn has_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    let parsed = parse_macro_input!(item as HandleAttributesInput);

    let operate_on = parsed.operate_on;
    let attributes = parsed.attributes;
    let mut result = MacroResult::default();

    let attributes_len = attributes.len();

    let attr_to_find_vars = indexed_name(quote::format_ident!("attr_to_find"), attributes_len);
    let found_vars = indexed_name(quote::format_ident!("found_vars"), attributes_len);

    let mut maybe_break = quote! {};
    //Add break; if only one attribute is passed in
    if attributes_len == 1 {
        maybe_break = quote! { break; };
    }

    //Check if attribute is present

    result.add(quote! {
        {
            #(
                let #attr_to_find_vars:syn::Attribute = syn::parse_quote! {
                    #attributes
                };
                let mut #found_vars = false;
            )*
            for attr in #operate_on.attrs.iter() {
                #(
                    if attr == &#attr_to_find_vars {
                        #found_vars = true;
                        #maybe_break
                    }
                )*
            }
            let mut found=true;
            #(
                if !#found_vars {
                    found=false;
                }
            )*
            found
        }
    });

    Ok(result.finalize().into())
}
#[derive(Debug)]
struct AttrWithUnknown {
    //Unknown coordinates
    ///Inside of final group/global
    unknown_coordinate: usize,
    unknown_group_coordinates: Vec<usize>,
    ///Used when unknown is inside of ident (and doesn't span entire literal) or literal
    partial_unknown_cords: Option<Range<usize>>,
    ///In the same group as the unknown
    ///
    /// In reverse order (right to left)
    tokens_after_unknown: Vec<proc_macro2::TokenTree>,
    before_unknown: String,
    after_unknown: String,
}

#[always_context]
impl AttrWithUnknown {
    fn new(attr: &syn::Attribute) -> anyhow::Result<Option<AttrWithUnknown>> {
        let stream = attr.to_token_stream();
        let string = stream.to_string();
        if let Some(pos) = string.find(*UNKNOWN) {
            //Get before and after unknown
            let before_unknown = string[..pos].to_string();
            let after_unknown = string[pos + UNKNOWN.len()..].to_string();
            //Get all tokens, coordinates, and tokens after unknown
            //later remove last coordinate and use it as `unknown_coordinate`
            let mut unknown_group_coordinates = vec![];

            struct DataRecursiveResult {
                partial_unknown_cords: Option<Range<usize>>,
                tokens_after: Option<Vec<proc_macro2::TokenTree>>,
            }

            ///# Return
            /// Tokens after unknown (in the same group) (not reversed)
            fn unknown_data_recursive(
                token_stream: proc_macro2::TokenStream,
                unknown_group_coordinates: &mut Vec<usize>,
            ) -> DataRecursiveResult {
                let mut tokens_after: Option<Vec<TokenTree>> = None;
                let mut partial_unknown_cords = None;

                for (index, token) in token_stream.into_iter().enumerate() {
                    if let Some(tokens_after) = &mut tokens_after {
                        //Unknown was found
                        tokens_after.push(token);
                    } else {
                        //Unknown not found yet
                        match token {
                            TokenTree::Group(group) => {
                                unknown_group_coordinates.push(index);
                                let result = unknown_data_recursive(
                                    group.stream(),
                                    unknown_group_coordinates,
                                );
                                if result.tokens_after.is_some() {
                                    return result;
                                } else {
                                    unknown_group_coordinates.pop();
                                }
                            }
                            TokenTree::Ident(ident) => {
                                let ident_str = ident.to_string();
                                let unknown_pos = ident_str.find(*UNKNOWN);

                                if let Some(u_pos) = unknown_pos {
                                    //Unknown found!
                                    tokens_after = Some(vec![]);
                                    unknown_group_coordinates.push(index);
                                    partial_unknown_cords = Some(Range {
                                        start: u_pos,
                                        end: u_pos + UNKNOWN.len(),
                                    });
                                }
                            }
                            TokenTree::Punct(_) => {}
                            TokenTree::Literal(literal) => {
                                let literal_str = literal.to_string();
                                let unknown_pos = literal_str.find(*UNKNOWN);

                                if let Some(u_pos) = unknown_pos {
                                    //Unknown found!
                                    tokens_after = Some(vec![]);
                                    unknown_group_coordinates.push(index);
                                    partial_unknown_cords = Some(Range {
                                        start: u_pos,
                                        end: u_pos + UNKNOWN.len(),
                                    });
                                }
                            }
                        }
                    }
                }

                DataRecursiveResult {
                    partial_unknown_cords,
                    tokens_after,
                }
            }

            let DataRecursiveResult {
                partial_unknown_cords,
                tokens_after,
            } = unknown_data_recursive(stream, &mut unknown_group_coordinates);

            let (token_after, unknown_coordinate) = if let Some(mut tokens_after) = tokens_after {
                //Reverse the tokens after unknown
                tokens_after.reverse();
                //Remove the last token (which is the unknown cord inside of last group)
                let unknown_coordinate = unknown_group_coordinates.pop().with_context(context!(
                    "No unknown coordinates, but tokens after are not None! | tokens_after: {:?}",
                    tokens_after
                ))?;
                (tokens_after, unknown_coordinate)
            } else {
                anyhow::bail!(
                    "Unknown not found in the attribute! Recursive call failed, but it shouldn't"
                );
            };

            return Ok(Some(AttrWithUnknown {
                before_unknown,
                after_unknown,
                unknown_coordinate,
                tokens_after_unknown: token_after,
                unknown_group_coordinates,
                partial_unknown_cords,
            }));
        }
        Ok(None)
    }

    fn get_unknown(&self, attr: &syn::Attribute) -> Option<proc_macro2::TokenStream> {
        //Check if start and end aligns with before and after unknown
        let attr_tokens = attr.to_token_stream();
        let attr_str = attr_tokens.to_string();

        //Speed up the process, check if the string starts and ends with tokens before and after the unknown
        if !(attr_str.starts_with(&self.before_unknown) && attr_str.ends_with(&self.after_unknown))
        {
            return None;
        }

        let mut current_tokens = attr_tokens;

        for group_index in self.unknown_group_coordinates.iter() {
            match current_tokens.into_iter().skip(*group_index).next() {
                Some(TokenTree::Group(group)) => {
                    current_tokens = group.stream();
                }
                i => {
                    panic!("Bad group index! Expected Group, got {i:?} | self: {self:?}");
                }
            }
        }

        //Get tokens at the unknown (and after)
        let mut unknown_tokens = current_tokens
            .into_iter()
            .skip(self.unknown_coordinate)
            .collect::<Vec<TokenTree>>();
        let unknown_tokens_len = unknown_tokens.len();

        //Remove last tokens after unknown
        if !self.tokens_after_unknown.is_empty() {
            unknown_tokens.drain(unknown_tokens_len - self.tokens_after_unknown.len()..);
        }

        Some(proc_macro2::TokenStream::from_iter(unknown_tokens))
    }
}

// fn find_unknown(attr_template:&syn::Attribute,attr:syn::)

//Allow for only one unknown inside of attribute
// __unknown__ - unknown mark
//Example: #[attribute__unknown__]
//Example: #[attri__unknown__bute]
//Example: #[__unknown__attribute]
//Example: #[attribute(__unknown__)]
//Example: #[attribute(name=__unknown__)]
//Example: #[attribute = __unknown__]
#[proc_macro]
#[macro_result]
pub fn get_attributes(item: TokenStream) -> anyhow::Result<TokenStream> {
    let parsed = parse_macro_input!(item as HandleAttributesInput);
    //The easiest way would be just turning attributes into a string and then parsing it
    //We would have to parse some parts into string anyway and this isn't performance critical

    let operate_on = parsed.operate_on;
    let mut attributes = parsed.attributes;
    let mut result = MacroResult::new();

    let unknown = {
        let mut unknown = None;
        for attr in attributes.iter() {
            if let Some(attr_with_unknown) = AttrWithUnknown::new(attr) {
                unknown = Some(attr_with_unknown);
                break;
            }
        }
        if let Some(u) = unknown {
            u
        } else {
            return anyhow::bail!("No unknown found in (to search for) attributes!");
        }
    };
}
