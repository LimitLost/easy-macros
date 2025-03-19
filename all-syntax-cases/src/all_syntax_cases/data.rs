use core::panic;
use std::collections::HashMap;

use quote::{ToTokens, quote};
use syn::{FieldValue, Signature, Token, TypeReference, punctuated::Punctuated};

pub struct InputSetup {
    generated_fn_prefix: String,
    additional_input_type: syn::Type,
    ///False by default
    system_functions_test: bool,
}

impl syn::parse::Parse for InputSetup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut generated_fn_prefix = None;
        let mut additional_input_type = None;
        let mut system_functions_test = false;

        while !input.is_empty() {
            if input.peek(Token![,]) {
                let _comma: Token![,] = input.parse()?;
            }
            let member: syn::Member = input.parse()?;
            let _colon: Token![:] = input.parse()?;
            match member {
                syn::Member::Named(ident) => {
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "generated_fn_prefix" => {
                            let lit_str: syn::LitStr = input.parse()?;
                            generated_fn_prefix = Some(lit_str.value());
                        }
                        "additional_input_type" => {
                            let ty: syn::Type = input.parse()?;
                            additional_input_type = Some(ty);
                        }
                        "system_functions_test" => {
                            let lit_bool: syn::LitBool = input.parse()?;
                            system_functions_test = lit_bool.value();
                        }
                        _ => {
                            panic!("Unknown member in setup: {}", ident_str);
                        }
                    }
                }
                syn::Member::Unnamed(_) => panic!("unnamed member not supported"),
            }
        }

        Ok(InputSetup {
            generated_fn_prefix: generated_fn_prefix
                .expect("generated_fn_prefix was not provided inside of setup => {...}"),
            additional_input_type: additional_input_type
                .expect("additional_input_type was not provided inside of setup => {...}"),
            system_functions_test,
        })
    }
}

pub struct Input {
    setup: InputSetup,
    default_cases: Punctuated<Signature, Token![;]>,
    special_cases: Punctuated<Signature, Token![;]>,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut setup = None;
        let mut default_cases = None;
        let mut special_cases = None;
        //Get Arms

        while !input.is_empty() {
            let pattern: syn::Path = input.parse()?;
            let _fat_arrow = input.parse::<Token![=>]>()?;
            let inside;
            let _braced = syn::braced!(inside in input);

            match pattern.to_token_stream().to_string().as_str() {
                "setup" => {
                    setup = Some(inside.parse::<InputSetup>()?);
                }
                "default_cases" => {
                    default_cases = Some(Punctuated::parse_terminated(&inside)?);
                }
                "special_cases" => {
                    special_cases = Some(Punctuated::parse_terminated(&inside)?);
                }
                p => {
                    panic!("Unknown arm: {}", p);
                }
            }
        }

        let setup = setup.expect("setup was not provided! Usage: setup => { <starting_point_type, generated_fn_prefix, additional_input_type> }");
        let default_cases = default_cases.expect(
            "default_cases was not provided! Usage: default_cases => { <function signatures> }",
        );
        let special_cases = special_cases.expect(
            "special_cases was not provided! Usage: special_cases => { <function signatures> }",
        );

        Ok(Input {
            setup,
            default_cases,
            special_cases,
        })
    }
}

pub enum ReferenceType {
    Mutable,
    Immutable,
    // ///Aka Dereference
    // Unbox,
}

pub struct EssentialFnData {
    input_types: Vec<syn::Type>,
    ident: syn::Ident,
    ///Used for showing errors (if false)
    used_at_least_once: bool,
}

fn type_equals_path_check(path1: &syn::Path, path2: &syn::Path) -> bool {
    if path1 == path2 {
        return true;
    }
    if let (Some(p1), Some(p2)) = (path1.segments.last(), path2.segments.last()) {
        if p1.ident != p2.ident {
            return false;
        }

        match (&p1.arguments, &p2.arguments) {
            (syn::PathArguments::None, syn::PathArguments::None) => true,
            (syn::PathArguments::AngleBracketed(p1), syn::PathArguments::AngleBracketed(p2)) => {
                if p1.args.len() != p2.args.len() {
                    return false;
                }

                for (arg1, arg2) in p1.args.iter().zip(p2.args.iter()) {
                    match (arg1, arg2) {
                        (
                            syn::GenericArgument::Lifetime(lifetime),
                            syn::GenericArgument::Lifetime(lifetime2),
                        ) => {
                            if lifetime != lifetime2 {
                                return false;
                            }
                        }
                        (syn::GenericArgument::Type(t1), syn::GenericArgument::Type(t2)) => {
                            if !type_equals(t1, t2) {
                                return false;
                            }
                        }
                        (syn::GenericArgument::Const(expr), syn::GenericArgument::Const(expr2)) => {
                            if expr != expr2 {
                                return false;
                            }
                        }
                        (
                            syn::GenericArgument::AssocType(_),
                            syn::GenericArgument::AssocType(_),
                        ) => {
                            panic!(
                                "all_syntax_cases Macro: AssocType type comparison not supported"
                            );
                        }
                        (
                            syn::GenericArgument::AssocConst(_),
                            syn::GenericArgument::AssocConst(_),
                        ) => {
                            panic!(
                                "all_syntax_cases Macro: AssocConst type comparison not supported"
                            );
                        }
                        (
                            syn::GenericArgument::Constraint(_),
                            syn::GenericArgument::Constraint(_),
                        ) => {
                            panic!(
                                "all_syntax_cases Macro: Constraint type comparison not supported"
                            );
                        }
                        _ => return false,
                    }
                }
                return true;
            }
            (syn::PathArguments::Parenthesized(_), syn::PathArguments::Parenthesized(_)) => {
                panic!("all_syntax_cases Macro: Parenthesized type comparison not supported");
            }
            _ => false,
        }
    } else {
        false
    }
}
///Checks if two types are equal, ignores `syn::` and `proc_macro2::` if present (checks only last part of path)
fn type_equals(ty1: &syn::Type, ty2: &syn::Type) -> bool {
    if ty1 == ty2 {
        return true;
    }
    match (ty1, ty2) {
        (syn::Type::Array(type_array), syn::Type::Array(type_array2)) => {
            //len eq check
            if type_array.len != type_array2.len {
                return false;
            }
            //Type eq check
            type_equals(&type_array.elem, &type_array2.elem)
        }
        (syn::Type::BareFn(_), syn::Type::BareFn(_)) => {
            panic!("all_syntax_cases Macro: BareFn type comparison not supported");
        }
        (syn::Type::Group(type_group), syn::Type::Group(type_group2)) => {
            type_equals(&type_group.elem, &type_group2.elem)
        }
        (syn::Type::ImplTrait(_), syn::Type::ImplTrait(_)) => {
            panic!("all_syntax_cases Macro: ImplTrait type comparison not supported");
        }
        (syn::Type::Infer(_), syn::Type::Infer(_)) => {
            panic!("all_syntax_cases Macro: Infer type comparison not supported");
        }
        (syn::Type::Macro(type_macro), syn::Type::Macro(type_macro2)) => {
            if type_macro.mac.tokens.to_string() != type_macro2.mac.tokens.to_string() {
                return false;
            }
            type_equals_path_check(&type_macro.mac.path, &type_macro2.mac.path)
        }
        (syn::Type::Never(_), syn::Type::Never(_)) => {
            panic!(
                "all_syntax_cases Macro: Never type comparison not supported (shouldn't fail anyway)"
            );
        }
        (syn::Type::Paren(type_paren), syn::Type::Paren(type_paren2)) => {
            type_equals(&type_paren.elem, &type_paren2.elem)
        }
        (syn::Type::Path(type_path), syn::Type::Path(type_path2)) => {
            if !type_equals_path_check(&type_path.path, &type_path2.path) {
                return false;
            }
            //Check if generics are equal
            if let (Some(type_path_qself), Some(type_path2_qself)) =
                (&type_path.qself, &type_path2.qself)
            {
                type_equals(&type_path_qself.ty, &type_path2_qself.ty)
            } else {
                unreachable!(
                    "TypePath paths are equal, generics are none, but somehow both types are not equal?"
                )
            }
        }
        (syn::Type::Ptr(type_ptr), syn::Type::Ptr(type_ptr2)) => {
            if type_ptr.mutability != type_ptr2.mutability {
                return false;
            }
            type_equals(&type_ptr.elem, &type_ptr2.elem)
        }
        (syn::Type::Reference(type_reference), syn::Type::Reference(type_reference2)) => {
            if type_reference.mutability != type_reference2.mutability {
                return false;
            }
            type_equals(&type_reference.elem, &type_reference2.elem)
        }
        (syn::Type::Slice(type_slice), syn::Type::Slice(type_slice2)) => {
            //Type eq check
            type_equals(&type_slice.elem, &type_slice2.elem)
        }
        (syn::Type::TraitObject(_), syn::Type::TraitObject(_)) => {
            panic!("all_syntax_cases Macro: TraitObject type comparison not supported");
        }
        (syn::Type::Tuple(type_tuple), syn::Type::Tuple(type_tuple2)) => {
            if type_tuple.elems.len() != type_tuple2.elems.len() {
                return false;
            }
            let iter = type_tuple.elems.iter().zip(type_tuple2.elems.iter());
            for (el, el2) in iter {
                if !type_equals(el, el2) {
                    return false;
                }
            }
            true
        }
        (syn::Type::Verbatim(token_stream), syn::Type::Verbatim(token_stream2)) => {
            token_stream.to_string() == token_stream2.to_string()
        }
        _ => false,
    }
}

impl EssentialFnData {
    pub fn new(sig: Signature) -> Self {
        let mut input_types = Vec::new();

        for input in sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => {
                    panic!("self arguments are not supported");
                }
                syn::FnArg::Typed(pat_type) => {
                    let ty = *pat_type.ty;
                    input_types.push(ty);
                }
            }
        }

        Self {
            input_types,
            ident: sig.ident,
            used_at_least_once: false,
        }
    }

    pub fn new_no_check(sig: Signature) -> Self {
        let mut input_types = Vec::new();

        for input in sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => {
                    panic!("self arguments are not supported");
                }
                syn::FnArg::Typed(pat_type) => {
                    let ty = *pat_type.ty;
                    input_types.push(ty);
                }
            }
        }

        Self {
            input_types,
            ident: sig.ident,
            used_at_least_once: true,
        }
    }

    ///Returns function call if all inputs are present
    pub fn all_inputs_check(
        &mut self,
        fields: &[syn::Field],
        before_dot: Option<&proc_macro2::TokenStream>,
        additional_input: (&syn::Ident, &syn::Type),
    ) -> Option<proc_macro2::TokenStream> {
        let (additional_input_ident, additional_input_ty) = additional_input;

        //Create reference list from required input types
        let mut reference_list = self.input_types.iter().enumerate().collect::<Vec<_>>();

        //Type used for creating final arguments list
        struct ResultArgData<'a> {
            ident: &'a syn::Ident,
            reference_ty: Option<ReferenceType>,
            ///From the `fields` argument side
            list: bool,
            ///Should add .clone() to the argument
            additional_ty: bool,
        }
        // key - real_index
        let mut result_args = HashMap::new();

        ///# Return
        /// `true` - found match
        fn fn_arg_ty_equals<'a>(
            reference_ty: &syn::Type,
            maybe_ty: &syn::Type,
            maybe_ident: &'a syn::Ident,
            real_index: &usize,
            result_args: &mut HashMap<usize, Vec<ResultArgData<'a>>>,
            additional_ty: bool,
        ) -> bool {
            if type_equals(reference_ty, maybe_ty) {
                let arg_data = result_args.entry(*real_index).or_insert(vec![]);
                arg_data.push(ResultArgData {
                    ident: maybe_ident,
                    reference_ty: None,
                    additional_ty,
                    list: false,
                });

                return true;
            } else {
                if let syn::Type::Reference(type_reference) = reference_ty {
                    if type_equals(&type_reference.elem, maybe_ty) {
                        let reference_ty = if type_reference.mutability.is_some() {
                            Some(ReferenceType::Mutable)
                        } else {
                            Some(ReferenceType::Immutable)
                        };
                        let arg_data = result_args.entry(*real_index).or_insert(vec![]);
                        arg_data.push(ResultArgData {
                            ident: maybe_ident,
                            reference_ty,
                            additional_ty,
                            list: false,
                        });

                        return true;
                    } else if let syn::Type::Path(ty_path) = maybe_ty {
                        fn handle_generic_ty<'a>(
                            args: &syn::PathArguments,
                            list: bool,
                            type_reference: &TypeReference,
                            result_args: &mut HashMap<usize, Vec<ResultArgData<'a>>>,
                            real_index: &usize,
                            maybe_ident: &'a syn::Ident,
                            additional_ty: bool,
                        ) -> bool {
                            //Get Type inside of <>
                            match args {
                                syn::PathArguments::None => {}
                                syn::PathArguments::AngleBracketed(
                                    angle_bracketed_generic_arguments,
                                ) => match angle_bracketed_generic_arguments.args.first() {
                                    Some(syn::GenericArgument::Type(ty)) => {
                                        if type_equals(&type_reference.elem, ty) {
                                            let reference_ty =
                                                if type_reference.mutability.is_some() {
                                                    Some(ReferenceType::Mutable)
                                                } else {
                                                    Some(ReferenceType::Immutable)
                                                };

                                            let arg_data =
                                                result_args.entry(*real_index).or_insert(vec![]);
                                            arg_data.push(ResultArgData {
                                                ident: maybe_ident,
                                                reference_ty,
                                                additional_ty,
                                                list,
                                            });

                                            return true;
                                        }
                                    }
                                    _ => {}
                                },
                                a => panic!(
                                    "all_syntax_cases Macro: Unsupported path arguments: {}",
                                    a.to_token_stream()
                                ),
                            }
                            false
                        }

                        let name_segment=ty_path.path.segments.last().expect("How the fuck this type doesn't have a single segment?! (should be unreachable)");
                        let ident_str = name_segment.ident.to_string();

                        let list = match ident_str.as_str() {
                            "Vec" | "Punctuated" => true,
                            _ => false,
                        };
                        //Handle Boxes, Vectors, Punctuated
                        match ident_str.as_str() {
                            "Vec" | "Box" | "Punctuated" => {
                                handle_generic_ty(
                                    &name_segment.arguments,
                                    list,
                                    type_reference,
                                    result_args,
                                    real_index,
                                    maybe_ident,
                                    additional_ty,
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
            false
        }

        let mut additional_argument_found = None;

        // Remove additional input type from reference list
        for (index, (real_index, ty)) in reference_list.iter().enumerate() {
            if fn_arg_ty_equals(
                ty,
                additional_input_ty,
                additional_input_ident,
                real_index,
                &mut result_args,
                true,
            ) {
                if additional_argument_found.is_none() {
                    // Additional input type argument should not repeat
                    additional_argument_found = Some(index);
                } else {
                    panic!(
                        "all_syntax_cases: additional input type should not repeat in function signature (it shouldn't be a type from syn or proc_macro2 libraries, or have the same name as type from any of those libraries)"
                    );
                }
            }
        }
        if let Some(index) = &additional_argument_found {
            reference_list.remove(*index);
        }

        //Search for all fields in reference list
        for field in fields.iter() {
            for (index, (real_index, ty)) in reference_list.iter().enumerate() {
                fn_arg_ty_equals(
                    ty,
                    &field.ty,
                    field.ident.as_ref().unwrap(),
                    real_index,
                    &mut result_args,
                    false,
                );
            }
        }

        let input_types_len = self.input_types.len();

        if result_args.len() == input_types_len {
            //All arguments, needed for function call, are present
            self.used_at_least_once = true;

            let mut result_args_vec = result_args.into_iter().collect::<Vec<_>>();

            //Sort arguments
            result_args_vec.sort_by(|a, b| a.0.cmp(&b.0));
            //Format arguments
            let before_dot = if let Some(before_dot) = before_dot {
                quote! {#before_dot.}
            } else {
                quote! {}
            };

            let multiple_calls_allowed = if additional_argument_found.is_some() {
                input_types_len == 2
            } else {
                input_types_len == 1
            };

            if !multiple_calls_allowed {
                //Check if there are more than one potential arguments per index
                for (_, arg) in result_args_vec.iter() {
                    if arg.len() > 1 {
                        //Too many potential arguments (we don't have any strategy to handle this)
                        //TODO Maybe in the future allow macro user to select how to handle this?
                        return None;
                    }
                }
            }
            let mut result_list_iterators: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut result_call_arguments: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();
            let mut additional_data_pos = 0;
            let mut additional_data_argument = None;

            for (real_pos, potential_args) in result_args_vec.into_iter() {
                for (index, arg) in potential_args.into_iter().enumerate() {
                    let arg_ident = arg.ident;
                    let mut before_dot = before_dot.clone();
                    let mut clone = quote! {};
                    if arg.additional_ty {
                        before_dot = Default::default();
                        clone = quote! { .clone() };
                    }
                    if arg.list {
                        let iter = match arg.reference_ty {
                            Some(ReferenceType::Mutable) => quote! { .iter_mut() },
                            Some(ReferenceType::Immutable) => quote! { .iter() },
                            None => unreachable!(
                                "all_syntax_cases Macro: List argument should have reference type (Unreachable)"
                            ),
                        };

                        result_list_iterators.push(quote! {
                            #before_dot #arg_ident #iter
                        });

                        continue;
                    }
                    let tokens = match arg.reference_ty {
                        Some(ReferenceType::Mutable) => {
                            quote! { &mut #before_dot #arg_ident #clone }
                        }
                        Some(ReferenceType::Immutable) => quote! { &#before_dot #arg_ident #clone },
                        None => quote! { #before_dot #arg_ident #clone },
                    };
                    if arg.additional_ty {
                        additional_data_pos = real_pos;
                        additional_data_argument = Some(tokens);
                    } else {
                        if let Some(v) = result_call_arguments.get_mut(index) {
                            v.push(tokens);
                        } else {
                            result_call_arguments.push(vec![tokens]);
                        }
                    }
                }
            }

            //Add additional data into every call
            if let Some(additional_data_argument) = additional_data_argument.clone() {
                for v in result_call_arguments.iter_mut() {
                    v.insert(additional_data_pos, additional_data_argument.clone());
                }
            }
            //Create function calls
            let fn_ident = &self.ident;

            //Handle list calls
            let list_calls_iter = result_list_iterators.iter().map(|iter| {
                if let Some(additional_arg) = additional_data_argument.clone() {
                    //More than one required args are not allowed yet
                    if additional_data_pos == 0 {
                        quote! {
                            for ____x in #iter{
                                #fn_ident(#additional_arg, ____x);
                            }
                        }
                    } else {
                        quote! {
                            for ____x in #iter{
                                #fn_ident(____x, #additional_arg);
                            }
                        }
                    }
                } else {
                    quote! {
                        for ____x in #iter{
                            #fn_ident(____x);
                        }
                    }
                }
            });

            let calls_iter = result_call_arguments.iter().map(|args| {
                quote! {
                    #fn_ident(#(#args),*);
                }
            });

            Some(quote! {
                #(#calls_iter)*
                #(#list_calls_iter)*
            })
        } else {
            None
        }
    }
}

pub struct MacroFnNames {
    pub item: syn::Ident,
    pub expr: syn::Ident,
    pub expr_option: syn::Ident,
    pub block: syn::Ident,
    pub stmt: syn::Ident,
    pub generics: syn::Ident,
    pub generic_param: syn::Ident,
    pub type_param_bound: syn::Ident,
    pub bound_lifetimes: syn::Ident,
    pub bound_lifetimes_option: syn::Ident,
    pub where_predicate: syn::Ident,
    pub impl_item: syn::Ident,
    pub item_mod_content: syn::Ident,
    pub fields: syn::Ident,
    pub trait_item: syn::Ident,
    pub fields_named: syn::Ident,
    pub option_box_expr: syn::Ident,
    pub pat: syn::Ident,
    pub option_else_expr: syn::Ident,
    pub arm: syn::Ident,
    pub angle_bracketed_generic_arguments: syn::Ident,
    pub range_limits: syn::Ident,
    pub field_value: syn::Ident,
    pub local_init: syn::Ident,
    pub option_local_init: syn::Ident,
    pub signature: syn::Ident,
    pub where_clause: syn::Ident,
    pub where_clause_option: syn::Ident,
    pub fn_arg: syn::Ident,
    pub variadic_pat: syn::Ident,
    pub variadic: syn::Ident,
    pub variadic_option: syn::Ident,
    pub field: syn::Ident,
    pub option_block: syn::Ident,
    pub option_eq_expr: syn::Ident,
    pub field_pat: syn::Ident,
    pub option_at_pat: syn::Ident,
    pub arm_guard: syn::Ident,
    pub option_angle_bracketed_generic_arguments: syn::Ident,
    pub generic_argument: syn::Ident,
    pub ty: syn::Ident,
    pub option_ty: syn::Ident,
    pub bare_fn_arg: syn::Ident,
    pub return_type: syn::Ident,
    pub variant: syn::Ident,
    pub foreign_item: syn::Ident,
    pub qself: syn::Ident,
    pub option_eq_type: syn::Ident,

    pub additional_input_name: syn::Ident,
}

impl MacroFnNames {
    pub fn new(fn_name_prefix: &str) -> Self {
        let item = quote::format_ident!("{}_item_handle", fn_name_prefix);
        let expr = quote::format_ident!("{}_expr_handle", fn_name_prefix);
        let expr_option = quote::format_ident!("{}_expr_option_handle", fn_name_prefix);
        let block = quote::format_ident!("{}_block_handle", fn_name_prefix);
        let stmt = quote::format_ident!("{}_stmt_handle", fn_name_prefix);
        let generics = quote::format_ident!("{}_generics_handle", fn_name_prefix);
        let generic_param = quote::format_ident!("{}_generic_param_handle", fn_name_prefix);
        let type_param_bound = quote::format_ident!("{}_type_param_bound_handle", fn_name_prefix);
        let bound_lifetimes = quote::format_ident!("{}_bound_lifetimes_handle", fn_name_prefix);
        let bound_lifetimes_option =
            quote::format_ident!("{}_bound_lifetimes_option_handle", fn_name_prefix);
        let where_predicate = quote::format_ident!("{}_where_predicate_handle", fn_name_prefix);
        let impl_item = quote::format_ident!("{}_impl_item_handle", fn_name_prefix);
        let item_mod_content = quote::format_ident!("{}_item_mod_content_handle", fn_name_prefix);
        let fields = quote::format_ident!("{}_fields_handle", fn_name_prefix);
        let trait_item = quote::format_ident!("{}_trait_item_handle", fn_name_prefix);
        let fields_named = quote::format_ident!("{}_fields_named_handle", fn_name_prefix);
        let option_box_expr = quote::format_ident!("{}_option_box_expr_handle", fn_name_prefix);
        let pat = quote::format_ident!("{}_pat_handle", fn_name_prefix);
        let option_else_expr = quote::format_ident!("{}_option_else_expr_handle", fn_name_prefix);
        let arm = quote::format_ident!("{}_arm_handle", fn_name_prefix);
        let angle_bracketed_generic_arguments = quote::format_ident!(
            "{}_angle_bracketed_generic_arguments_handle",
            fn_name_prefix
        );
        let range_limits = quote::format_ident!("{}_range_limits_handle", fn_name_prefix);
        let field_value = quote::format_ident!("{}_field_value_handle", fn_name_prefix);
        let local_init = quote::format_ident!("{}_local_init_handle", fn_name_prefix);
        let option_local_init = quote::format_ident!("{}_option_local_init_handle", fn_name_prefix);
        let signature = quote::format_ident!("{}_signature_handle", fn_name_prefix);
        let where_clause = quote::format_ident!("{}_where_clause_handle", fn_name_prefix);
        let where_clause_option =
            quote::format_ident!("{}_where_clause_option_handle", fn_name_prefix);
        let fn_arg = quote::format_ident!("{}_fn_arg_handle", fn_name_prefix);
        let variadic_pat = quote::format_ident!("{}_variadic_pat_handle", fn_name_prefix);
        let variadic = quote::format_ident!("{}_variadic_handle", fn_name_prefix);
        let variadic_option = quote::format_ident!("{}_variadic_option_handle", fn_name_prefix);
        let field = quote::format_ident!("{}_field_handle", fn_name_prefix);
        let option_block = quote::format_ident!("{}_option_block_handle", fn_name_prefix);
        let option_eq_expr = quote::format_ident!("{}_option_eq_expr_handle", fn_name_prefix);
        let field_pat = quote::format_ident!("{}_field_pat_handle", fn_name_prefix);
        let option_at_pat = quote::format_ident!("{}_option_at_pat_handle", fn_name_prefix);
        let arm_guard = quote::format_ident!("{}_arm_guard_handle", fn_name_prefix);
        let option_angle_bracketed_generic_arguments = quote::format_ident!(
            "{}_option_angle_bracketed_generic_arguments_handle",
            fn_name_prefix
        );
        let generic_argument = quote::format_ident!("{}_generic_argument_handle", fn_name_prefix);
        let ty = quote::format_ident!("{}_ty_handle", fn_name_prefix);
        let option_ty = quote::format_ident!("{}_option_ty_handle", fn_name_prefix);
        let bare_fn_arg = quote::format_ident!("{}_bare_fn_arg_handle", fn_name_prefix);
        let return_type = quote::format_ident!("{}_return_type_handle", fn_name_prefix);
        let variant = quote::format_ident!("{}_variant_handle", fn_name_prefix);
        let foreign_item = quote::format_ident!("{}_foreign_item_handle", fn_name_prefix);
        let qself = quote::format_ident!("{}_qself_handle", fn_name_prefix);
        let option_eq_type = quote::format_ident!("{}_option_eq_type_handle", fn_name_prefix);

        let additional_input_name = quote::format_ident!("__additional_input");

        Self {
            item,
            expr,
            expr_option,
            block,
            stmt,
            generics,
            generic_param,
            type_param_bound,
            bound_lifetimes,
            bound_lifetimes_option,
            where_predicate,
            impl_item,
            item_mod_content,
            fields,
            trait_item,
            fields_named,
            option_box_expr,
            pat,
            option_else_expr,
            arm,
            angle_bracketed_generic_arguments,
            range_limits,
            field_value,
            local_init,
            option_local_init,
            signature,
            where_clause,
            where_clause_option,
            fn_arg,
            variadic_pat,
            variadic,
            variadic_option,
            field,
            option_block,
            option_eq_expr,
            field_pat,
            option_at_pat,
            arm_guard,
            option_angle_bracketed_generic_arguments,
            generic_argument,
            ty,
            option_ty,
            bare_fn_arg,
            return_type,
            variant,
            foreign_item,
            qself,
            option_eq_type,

            additional_input_name,
        }
    }
}

pub struct MacroData {
    pub fn_names: MacroFnNames,
    pub generated_fn_prefix: String,
    pub additional_input_ty: syn::Type,
    pub default_functions: Vec<EssentialFnData>,
    pub special_functions: Vec<EssentialFnData>,
    ///Special calls should happen after the default calls
    pub system_functions: Vec<EssentialFnData>,
}

impl MacroData {
    pub fn new(macro_input: Input) -> Self {
        let Input {
            setup,
            default_cases,
            special_cases,
        } = macro_input;

        let fn_names = MacroFnNames::new(&setup.generated_fn_prefix);
        let additional_input_ty = setup.additional_input_type;
        let fn_name_prefix = setup.generated_fn_prefix;

        //Create function data
        let mut default_functions = Vec::new();
        for sig in default_cases.iter() {
            default_functions.push(EssentialFnData::new(sig.clone()));
        }

        let mut special_functions = Vec::new();
        for sig in special_cases.iter() {
            special_functions.push(EssentialFnData::new(sig.clone()));
        }

        struct SystemNewFn(fn(syn::Signature) -> EssentialFnData);

        let mut system_new_fn = SystemNewFn(EssentialFnData::new_no_check);

        //Activate checks back for testing
        if setup.system_functions_test {
            system_new_fn = SystemNewFn(EssentialFnData::new);
        }

        let mut system_functions = Vec::new();
        let MacroFnNames {
            item,
            expr,
            expr_option,
            block,
            stmt,
            additional_input_name,
            generic_param,
            generics,
            type_param_bound,
            bound_lifetimes,
            bound_lifetimes_option,
            where_predicate,
            impl_item,
            item_mod_content,
            fields,
            trait_item,
            fields_named,
            option_box_expr,
            pat,
            option_else_expr,
            arm,
            angle_bracketed_generic_arguments,
            range_limits,
            field_value,
            local_init,
            option_local_init,
            signature,

            where_clause,
            where_clause_option,
            fn_arg,
            variadic_pat,
            variadic,
            variadic_option,
            field,
            option_block,
            option_eq_expr,
            field_pat,
            option_at_pat,
            arm_guard,
            option_angle_bracketed_generic_arguments,
            generic_argument,
            ty,
            option_ty,
            bare_fn_arg,
            return_type,
            variant,
            foreign_item,
            qself,
            option_eq_type,
        } = &fn_names;
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #item(item: &mut Item, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #expr(expr: &mut Expr, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #expr_option(expr_option: &mut Option<Expr>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #block(block: &mut Block, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #stmt(stmt: &mut Stmt, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #generic_param(generic_param: &mut GenericParam, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #generics(generics: &mut Generics, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #type_param_bound(type_param_bound: &mut TypeParamBound, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #bound_lifetimes(bound_lifetimes: &mut BoundLifetimes, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #bound_lifetimes_option(bound_lifetimes_option: &mut Option<BoundLifetimes>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #where_predicate(where_predicate: &mut WherePredicate, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #impl_item(impl_item: &mut ImplItem, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #item_mod_content(item_mod_content: &mut Option<(syn::token::Brace, Vec<Item>)>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #fields(fields: &mut Fields, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #trait_item(trait_item: &mut TraitItem, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #fields_named(fields_named: &mut FieldsNamed, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #option_box_expr(option_box_expr: &mut Option<Box<Expr>>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #pat(pat: &mut Pat, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #option_else_expr(option_else_expr: &mut Option<(Token![else], Box<Expr>)>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #arm(arm: &mut Arm, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #angle_bracketed_generic_arguments(angle_bracketed_generic_arguments: &mut AngleBracketedGenericArguments, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #range_limits(range_limits: &mut RangeLimits, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #field_value(field_value: &mut FieldValue, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #local_init(local_init: &mut LocalInit, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #option_local_init(option_local_init: &mut Option<LocalInit>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #signature(signature: &mut Signature, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #where_clause(where_clause: &mut WhereClause, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #where_clause_option(where_clause_option: &mut Option<WhereClause>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #fn_arg(fn_arg: &mut FnArg, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #variadic_pat(variadic_pat: &mut Option<(Box<Pat>, Token![:])>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #variadic(variadic: &mut Variadic, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #variadic_option(variadic_option: &mut Option<Variadic>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #field(field: &mut Field, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #option_block(option_block: &mut Option<Block>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #option_eq_expr(option_eq_expr: &mut Option<(Token![=], Expr)>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #field_pat(field_pat: &mut FieldPat, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #option_at_pat(option_at_pat: &mut Option<(Token![@], Pat)>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #arm_guard(arm_guard: &mut Option<(Token![if], Box<Expr>)>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #option_angle_bracketed_generic_arguments(option_angle_bracketed_generic_arguments: &mut Option<AngleBracketedGenericArguments>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #generic_argument(generic_argument: &mut GenericArgument, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #ty(ty: &mut Type, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #option_ty(option_ty: &mut Option<Type>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #bare_fn_arg(bare_fn_arg: &mut BareFnArg, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #return_type(return_type: &mut ReturnType, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #variant(variant: &mut Variant, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #foreign_item(foreign_item: &mut ForeignItem, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #qself(qself: &mut QSelf, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #option_eq_type(option_eq_type: &mut Option<(Token![=], Type)>, #additional_input_name: #additional_input_ty)
        }));

        Self {
            fn_names,
            generated_fn_prefix: fn_name_prefix,
            additional_input_ty,
            default_functions,
            special_functions,
            system_functions,
        }
    }
}
#[test]
fn type_equals_test() {}

#[test]
fn essential_fn_checks_test() {
    struct AdditionalInput;

    let additional_input_base: (syn::Ident, syn::Type) = (
        quote::format_ident!("__additional_input"),
        syn::parse_quote! {AdditionalInput},
    );
    let additional_input = (&additional_input_base.0, &additional_input_base.1);

    let mut fn_data1 = EssentialFnData::new(syn::parse_quote! {
        fn example_fn1(a: &mut syn::Item)
    });

    let input_fields1 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: &mut syn::Item,
                b: &mut syn::Expr,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };

    let input_fields2 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: syn::Item,
                b: syn::Expr,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };

    let input_fields3 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: Box<syn::Item>,
                b: Box<syn::Expr>,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };

    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields1, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(x);
            }
            .to_string()
        )
    );
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields2, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x);
            }
            .to_string()
        )
    );
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields3, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x);
            }
            .to_string()
        )
    );
}
