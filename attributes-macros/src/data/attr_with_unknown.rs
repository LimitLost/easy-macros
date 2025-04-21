use always_context::always_context;
use anyhow::Context;
use helpers_context::{context, context_internal2};
use lazy_static::lazy_static;
use proc_macro2::TokenTree;
use quote::ToTokens;

lazy_static! {
    static ref UNKNOWN: &'static str = "__unknown__";
    static ref UNKNOWN_REGEX: regex::Regex =
        regex::Regex::new(&regex::escape(*UNKNOWN)).expect("Failed to create regex for unknown");
}

#[derive(Debug)]
#[allow(dead_code)]
///Real implementation is in attributes crate
pub struct AttrWithUnknown {
    //Unknown coordinates
    ///Inside of final group/global
    unknown_coordinate: usize,
    unknown_group_coordinates: Vec<usize>,
    ///Inside of ident or literal
    partial_unknown_cords: PartialUnknownPos,
    ///In the same group as the unknown
    ///
    /// In reverse order (right to left)
    tokens_after_unknown: Vec<proc_macro2::TokenTree>,
    before_unknown: String,
    after_unknown: String,
}
#[derive(Debug)]
#[allow(dead_code)]
///Real implementation is in attributes crate
struct PartialUnknownPos {
    skip_start: usize,
    skip_end: usize,
}

#[always_context]
impl AttrWithUnknown {
    pub fn new(attr: &syn::Attribute) -> anyhow::Result<Option<AttrWithUnknown>> {
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
                partial_unknown_cords: PartialUnknownPos,
                tokens_after: Vec<proc_macro2::TokenTree>,
            }

            ///# Return
            /// Tokens after unknown (in the same group) (not reversed)
            fn unknown_data_recursive(
                token_stream: proc_macro2::TokenStream,
                unknown_group_coordinates: &mut Vec<usize>,
            ) -> Option<DataRecursiveResult> {
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
                                if result.is_some() {
                                    return result;
                                } else {
                                    unknown_group_coordinates.pop();
                                }
                            }
                            TokenTree::Ident(ident) => {
                                let ident_str = ident.to_string();
                                let unknown_pos = UNKNOWN_REGEX.find(&ident_str);

                                if let Some(u_pos) = unknown_pos {
                                    //Unknown found!
                                    tokens_after = Some(vec![]);
                                    unknown_group_coordinates.push(index);
                                    partial_unknown_cords = Some(PartialUnknownPos {
                                        skip_start: u_pos.start(),
                                        skip_end: ident_str.len() - u_pos.end(),
                                    });
                                }
                            }
                            TokenTree::Punct(_) => {}
                            TokenTree::Literal(literal) => {
                                let literal_str = literal.to_string();
                                let unknown_pos = UNKNOWN_REGEX.find(&literal_str);

                                if let Some(u_pos) = unknown_pos {
                                    //Unknown found!
                                    tokens_after = Some(vec![]);
                                    unknown_group_coordinates.push(index);
                                    partial_unknown_cords = Some(PartialUnknownPos {
                                        skip_start: u_pos.start(),
                                        skip_end: literal_str.len() - u_pos.end(),
                                    });
                                }
                            }
                        }
                    }
                }

                if let (Some(partial_unknown_cords), Some(tokens_after)) =
                    (partial_unknown_cords, tokens_after)
                {
                    Some(DataRecursiveResult {
                        partial_unknown_cords,
                        tokens_after,
                    })
                } else {
                    None
                }
            }

            let data_recursive_result =
                unknown_data_recursive(stream, &mut unknown_group_coordinates);

            let (token_after, unknown_coordinate, partial_unknown_cords) = if let Some(
                DataRecursiveResult {
                    partial_unknown_cords,
                    mut tokens_after,
                },
            ) =
                data_recursive_result
            {
                //Reverse the tokens after unknown
                tokens_after.reverse();
                //Remove the last token (which is the unknown cord inside of last group)
                let unknown_coordinate = unknown_group_coordinates.pop().with_context(context!(
                    "No unknown coordinates, but tokens after are not None! | tokens_after: {:?}",
                    tokens_after
                ))?;
                (tokens_after, unknown_coordinate, partial_unknown_cords)
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
}
