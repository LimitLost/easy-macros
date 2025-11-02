//! Internal implementation details. Not intended for direct use.
//!
//! This module contains the low-level attribute parsing machinery used by the public macros.
//! Use [`get_attributes!`] and [`fields_get_attributes!`] instead.

use always_context::always_context;
use anyhow::Context;
use helpers::context;
use lazy_static::lazy_static;
use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::{Ident, LitStr};

lazy_static! {
    pub(crate) static ref UNKNOWN: &'static str = "__unknown__";
    pub(crate) static ref UNKNOWN_REGEX: regex::Regex =
        regex::Regex::new(&regex::escape(*UNKNOWN)).expect("Failed to create regex for unknown");
}

/// Low-level attribute parser used internally by the attribute extraction macros.
///
/// **You should not use this type directly.** Use [`get_attributes!`] or [`fields_get_attributes!`] instead.
///
/// This type is exported for technical reasons (used by the proc macro crate) but is not
/// part of the stable API. Implementation details may change without notice.
#[derive(Debug)]
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
            let before_unknown = string.get(..pos)?.to_string();
            let after_unknown = string.get(pos + UNKNOWN.len()..)?.to_string();

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

    pub fn get_unknown(
        &self,
        attr: &syn::Attribute,
    ) -> anyhow::Result<Option<proc_macro2::TokenStream>> {
        //Check if start and end aligns with before and after unknown
        let attr_tokens = attr.to_token_stream();
        let attr_str = attr_tokens.to_string();

        //Speed up the process, check if the string starts and ends with tokens before and after the unknown
        if !(attr_str.starts_with(&self.before_unknown) && attr_str.ends_with(&self.after_unknown))
        {
            return Ok(None);
        }

        let mut current_tokens = attr_tokens;

        for group_index in self.unknown_group_coordinates.iter() {
            match current_tokens.into_iter().nth(*group_index) {
                Some(TokenTree::Group(group)) => {
                    current_tokens = group.stream();
                }
                i => {
                    anyhow::bail!("Bad group index! Expected Group, got {i:?} | self: {self:?}");
                }
            }
        }

        //Get tokens at the unknown (and after)
        let mut unknown_tokens = current_tokens
            .into_iter()
            .skip(self.unknown_coordinate)
            .collect::<Vec<TokenTree>>();
        let unknown_tokens_len = unknown_tokens.len();

        // Remove tokens after unknown
        if !self.tokens_after_unknown.is_empty() {
            unknown_tokens.drain(unknown_tokens_len - self.tokens_after_unknown.len()..);
        }

        // Handle partial_unknown_cords
        {
            let partial_unknown_cords = &self.partial_unknown_cords;

            //Remove before unknown in ident/literal
            if partial_unknown_cords.skip_start != 0 {
                let mut remove_first = false;
                match unknown_tokens.first_mut() {
                    Some(TokenTree::Ident(ident)) => {
                        let ident_str = ident.to_string();
                        let unknown_replacement = &ident_str[partial_unknown_cords.skip_start..];

                        if unknown_replacement.is_empty() {
                            remove_first = true;
                        } else {
                            *ident = Ident::new(unknown_replacement, ident.span());
                        }
                    }
                    Some(TokenTree::Literal(literal)) => {
                        let lit_str = syn::parse2::<LitStr>(proc_macro2::TokenStream::from(
                            TokenTree::Literal(literal.clone()),
                        ))?;
                        let literal_str = lit_str.value();
                        let unknown_replacement = &literal_str[partial_unknown_cords.skip_start..];

                        if unknown_replacement.is_empty() {
                            remove_first = true;
                        } else {
                            *literal = proc_macro2::Literal::string(unknown_replacement);
                        }
                    }
                    Some(i) => anyhow::bail!(
                        "Expected ident or literal (for removing text before unknown), got {i}"
                    ),
                    None => {
                        anyhow::bail!(
                            "Unknown tokens is empty while looking for partial unknown! | self: {self:?}"
                        );
                    }
                }

                if remove_first {
                    unknown_tokens.remove(0);
                }
            }
            //Remove after unknown in ident/literal
            if partial_unknown_cords.skip_end != 0 {
                let mut remove_last = false;
                match unknown_tokens.last_mut() {
                    Some(TokenTree::Ident(ident)) => {
                        let ident_str = ident.to_string();
                        let unknown_replacement =
                            &ident_str[0..ident_str.len() - partial_unknown_cords.skip_end];

                        if unknown_replacement.is_empty() {
                            remove_last = true;
                        } else {
                            *ident = Ident::new(unknown_replacement, ident.span());
                        }
                    }
                    Some(TokenTree::Literal(literal)) => {
                        let lit_str = syn::parse2::<LitStr>(proc_macro2::TokenStream::from(
                            TokenTree::Literal(literal.clone()),
                        ))?;
                        let literal_str = lit_str.value();
                        let unknown_replacement =
                            &literal_str[0..literal_str.len() - partial_unknown_cords.skip_end];

                        if unknown_replacement.is_empty() {
                            remove_last = true;
                        } else {
                            *literal = proc_macro2::Literal::string(unknown_replacement);
                        }
                    }
                    Some(i) => anyhow::bail!(
                        "Expected ident or literal (for removing text after unknown), got {i}"
                    ),
                    None => {
                        anyhow::bail!(
                            "Unknown tokens is empty while looking for partial unknown! | self: {self:?}"
                        );
                    }
                }

                if remove_last {
                    unknown_tokens.pop();
                }
            }
        }

        Ok(Some(proc_macro2::TokenStream::from_iter(unknown_tokens)))
    }
}
