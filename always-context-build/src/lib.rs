use std::{io::Write, path::Path};

use always_context::always_context;
use anyhow::Context;
use helpers_context::context;
use proc_macro2::{LineColumn, TokenStream};
use quote::ToTokens;
use syn::{Meta, Type, spanned::Spanned};

#[derive(Debug, Default)]
struct FileUpdates {
    ///Where to add `#[always_context]`
    updates: Vec<LineColumn>,
}
///Returns `true` if the type is `anyhow::Result`
fn anyhow_result_check(ty: &Type) -> bool {
    if let Type::Path(ty) = ty {
        let mut segments = ty.path.segments.iter();
        if let Some(segment) = segments.next() {
            if segment.ident != "anyhow" {
                return false;
            }
        } else {
            return false;
        }
        if let Some(segment) = segments.next() {
            if segment.ident == "Result" {
                return true;
            }
        }
    }
    false
}

fn has_always_context(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if let Meta::Path(path) = &attr.meta {
            let path_str = path
                .to_token_stream()
                .to_string()
                .replace(|c: char| c.is_whitespace(), "");
            if let "always_context" | "always_context::always_context" = path_str.as_str() {
                return true;
            }
        }
    }
    false
}
///Returns `true` if the function has `anyhow::Result` return type and does not have `#[always_context]` attribute
#[always_context]
fn handle_fn(sig: &syn::Signature, attrs: &[syn::Attribute]) -> anyhow::Result<bool> {
    match &sig.output {
        syn::ReturnType::Default => {
            //No anyhow::Result
            Ok(false)
        }
        syn::ReturnType::Type(_, ty) => {
            if anyhow_result_check(ty) && !has_always_context(attrs) {
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

#[always_context]
fn handle_item(item: &syn::Item, file_updates: &mut Option<FileUpdates>) -> anyhow::Result<()> {
    match item {
        syn::Item::Fn(item_fn) => {
            if handle_fn(
                #[context(tokens)]
                &item_fn.sig,
                #[context(tokens_vec)]
                &item_fn.attrs,
            )? {
                let updates = file_updates.get_or_insert_default();

                updates.updates.push(item_fn.span().start());
            }
        }
        syn::Item::ForeignMod(item_foreign_mod) => {
            for item in item_foreign_mod.items.iter() {
                if let syn::ForeignItem::Fn(foreign_item_fn) = item {
                    if handle_fn(
                        #[context(tokens)]
                        &foreign_item_fn.sig,
                        #[context(tokens_vec)]
                        &foreign_item_fn.attrs,
                    )? {
                        let updates = file_updates.get_or_insert_default();

                        updates.updates.push(foreign_item_fn.span().start());
                    }
                }
            }
        }
        syn::Item::Trait(item_trait) => {
            if !has_always_context(&item_trait.attrs) {
                let updates = file_updates.get_or_insert_default();

                updates.updates.push(item_trait.span().start());
            }
        }
        syn::Item::Impl(item_impl) => {
            if !has_always_context(&item_impl.attrs) {
                let updates = file_updates.get_or_insert_default();

                updates.updates.push(item_impl.span().start());
            }
        }
        syn::Item::Mod(item_mod) => {
            if let Some((_, content)) = item_mod.content.as_ref() {
                for item in content.iter() {
                    handle_item(
                        #[context(tokens)]
                        item,
                        file_updates,
                    )?;
                }
            }
        }
        _ => {
            // Ignore other items
        }
    }
    Ok(())
}
/// # Inputs
/// `line` - 0 indexed
#[always_context]
fn line_pos(haystack: &str, line: usize) -> anyhow::Result<usize> {
    let mut regex_str = "^".to_string();
    for _ in 0..line {
        regex_str.push_str(r".*((\r\n)|\r|\n)");
    }
    let regex = regex::Regex::new(&regex_str)?;

    let found = regex
        .find_at(haystack, 0)
        .with_context(context!("Finding line failed! | Regex: {:?}", regex))?;

    Ok(found.end())
}
#[always_context]
fn handle_file(file_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let file_path = file_path.as_ref();
    // Check if the file is a rust file
    match file_path.extension() {
        Some(ext) if ext == "rs" => {}
        _ => return Ok(()),
    }

    // Read the file
    let mut contents = std::fs::read_to_string(file_path)?;
    //Operate on syn::File
    let mut file_updates: Option<FileUpdates> = None;
    let file = match syn::parse_file(&contents) {
        Ok(file) => file,
        Err(_) => {
            //Ignore files with errors
            return Ok(());
        }
    };

    for item in file.items.into_iter() {
        handle_item(
            #[context(tokens)]
            &item,
            &mut file_updates,
        )?;
    }

    // Update the file (if needed)
    if let Some(updates) = file_updates {
        let mut updates = updates.updates;
        //Sort our lines and reverse them
        updates.sort_by(|a, b| a.line.cmp(&b.line));
        updates.reverse();

        //Uses span position info to add #[always_context] to every item on the list
        for start_pos in updates.into_iter() {
            //1 indexed
            let line = start_pos.line;
            //Find position based on line
            let line_bytes_end = line_pos(&contents, line - 1)?;

            contents.insert_str(line_bytes_end, "#[always_context]\r\n");
        }

        let mut file = std::fs::File::create(file_path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    Ok(())
}

#[always_context]
fn handle_dir(
    dir: impl AsRef<Path>,
    ignore_list: &[regex::Regex],
    base_path_len_bytes: usize,
) -> anyhow::Result<()> {
    // Get all files in the src directory
    let files = std::fs::read_dir(dir.as_ref())?;
    // Iterate over all files
    'entries: for entry in files {
        #[no_context_inputs]
        let entry = entry?;

        // Get the file path
        let entry_path = entry.path();

        //Ignore list check
        for r in ignore_list.iter() {
            let path_str = entry_path.display().to_string();

            if r.is_match(&path_str[base_path_len_bytes..]) {
                // Ignore this entry
                continue 'entries;
            }
        }

        let file_type = entry.file_type()?;
        if file_type.is_file() {
            handle_file(&entry_path)?;
        } else if file_type.is_dir() {
            // If the file is a directory, call this function recursively
            handle_dir(&entry_path, ignore_list, base_path_len_bytes)?;
        }
    }

    Ok(())
}

#[always_context]
/// Build function that adds `#[always_context]` attribute to every function with `anyhow::Result` return type and every `trait` and `impl` block.
///
/// To every rust file in `src` directory.
///
/// # Arguments
///
/// `ignore_list` - A list of regex patterns to ignore. The patterns are used on the file path. Path is ignored if match found.
///
fn build_result(ignore_list: &[regex::Regex]) -> anyhow::Result<()> {
    // Get the current directory
    let current_dir = std::env::current_dir()?;
    let base_path_len_bytes = current_dir.display().to_string().len();
    // Get the src directory
    let src_dir = current_dir.join("src");

    handle_dir(&src_dir, ignore_list, base_path_len_bytes)?;

    Ok(())
}
/// Build function that adds `#[always_context]` attribute to every function with `anyhow::Result` return type and every `trait` and `impl` block.
///
/// To every rust file in `src` directory.
///
/// Panics on error. Use `build_result()` for error handling.
///
/// # Arguments
///
/// `ignore_list` - A list of regex patterns to ignore. The patterns are used on the file path. Path is ignored if match found.
///
pub fn build(ignore_list: &[regex::Regex]) {
    if let Err(err) = build_result(ignore_list) {
        panic!(
            "Always Context Build Error: {}\r\n\r\nDebug Info:\r\n\r\n{:?}",
            err, err
        );
    }
}
