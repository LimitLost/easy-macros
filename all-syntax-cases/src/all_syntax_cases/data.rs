use core::panic;
use std::collections::HashMap;

use quote::{ToTokens, quote};
use syn::{Signature, Token, TypeReference, punctuated::Punctuated};

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

pub struct AttrsSignature {
    attrs: Vec<syn::Attribute>,
    sig: syn::Signature,
}

impl syn::parse::Parse for AttrsSignature {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let sig: syn::Signature = input.parse()?;
        Ok(AttrsSignature { attrs, sig })
    }
}

impl AttrsSignature {
    fn after_system(&self) -> bool {
        for attr in self.attrs.iter() {
            if attr.path().is_ident("after_system") {
                return true;
            }
        }
        false
    }
}

pub struct Input {
    setup: InputSetup,
    default_cases: Punctuated<AttrsSignature, Token![;]>,
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

        let setup = setup.expect("setup was not provided! Usage: setup => { <generated_fn_prefix, additional_input_type> }");
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

#[derive(Debug, PartialEq, Eq)]
pub enum AdditionalType {
    Reference,
    ///Should add .clone() to the argument
    NoReference,
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
                true
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
                true
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

pub fn additional_type(active: bool, ty: &syn::Type) -> Option<AdditionalType> {
    if active {
        if let syn::Type::Reference(_) = ty {
            Some(AdditionalType::Reference)
        } else {
            Some(AdditionalType::NoReference)
        }
    } else {
        None
    }
}

fn additional_type_no_ref(active: bool) -> Option<AdditionalType> {
    if active {
        Some(AdditionalType::NoReference)
    } else {
        None
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
            additional_ty: Option<AdditionalType>,
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
                let arg_data = result_args.entry(*real_index).or_default();
                arg_data.push(ResultArgData {
                    ident: maybe_ident,
                    reference_ty: None,
                    additional_ty: additional_type(additional_ty, maybe_ty),
                    list: false,
                });

                return true;
            } else if let syn::Type::Reference(type_reference) = reference_ty {
                if type_equals(&type_reference.elem, maybe_ty) {
                    let reference_ty = if type_reference.mutability.is_some() {
                        Some(ReferenceType::Mutable)
                    } else {
                        Some(ReferenceType::Immutable)
                    };
                    let arg_data = result_args.entry(*real_index).or_default();
                    arg_data.push(ResultArgData {
                        ident: maybe_ident,
                        reference_ty,
                        additional_ty: additional_type(additional_ty, maybe_ty),
                        list: false,
                    });

                    return true;
                } else {
                    fn handle_path<'a>(
                        maybe_ty: &syn::Type,
                        type_reference: &TypeReference,
                        result_args: &mut HashMap<usize, Vec<ResultArgData<'a>>>,
                        real_index: &usize,
                        maybe_ident: &'a syn::Ident,
                        additional_ty: bool,
                        current_reference_ty: Option<Option<ReferenceType>>,
                    ) -> bool {
                        if let syn::Type::Path(ty_path) = maybe_ty {
                            #[allow(clippy::too_many_arguments)]
                            fn handle_generic_ty<'a>(
                                args: &syn::PathArguments,
                                list: bool,
                                type_reference: &TypeReference,
                                result_args: &mut HashMap<usize, Vec<ResultArgData<'a>>>,
                                real_index: &usize,
                                maybe_ident: &'a syn::Ident,
                                additional_ty: bool,
                                current_reference_ty: Option<Option<ReferenceType>>,
                            ) -> bool {
                                //Get Type inside of <>
                                match args {
                                    syn::PathArguments::None => {}
                                    syn::PathArguments::AngleBracketed(
                                        angle_bracketed_generic_arguments,
                                    ) => {
                                        if let Some(syn::GenericArgument::Type(ty)) =
                                            angle_bracketed_generic_arguments.args.first()
                                        {
                                            if type_equals(&type_reference.elem, ty) {
                                                let reference_ty =
                                                    if let Some(current_reference_ty) =
                                                        current_reference_ty
                                                    {
                                                        current_reference_ty
                                                    } else if type_reference.mutability.is_some() {
                                                        Some(ReferenceType::Mutable)
                                                    } else {
                                                        Some(ReferenceType::Immutable)
                                                    };

                                                let arg_data =
                                                    result_args.entry(*real_index).or_default();
                                                arg_data.push(ResultArgData {
                                                    ident: maybe_ident,
                                                    reference_ty,
                                                    additional_ty: additional_type_no_ref(
                                                        additional_ty,
                                                    ),
                                                    list,
                                                });

                                                return true;
                                            }
                                        }
                                    }
                                    a => panic!(
                                        "all_syntax_cases Macro: Unsupported path arguments: {}",
                                        a.to_token_stream()
                                    ),
                                }
                                false
                            }

                            let name_segment=ty_path.path.segments.last().expect("How the fuck this type doesn't have a single segment?! (should be unreachable)");
                            let ident_str = name_segment.ident.to_string();

                            let list = matches!(ident_str.as_str(), "Vec" | "Punctuated");
                            //Handle Boxes, Vectors, Punctuated
                            match ident_str.as_str() {
                                "Vec" | "Box" | "Punctuated" => {
                                    if handle_generic_ty(
                                        &name_segment.arguments,
                                        list,
                                        type_reference,
                                        result_args,
                                        real_index,
                                        maybe_ident,
                                        additional_ty,
                                        current_reference_ty,
                                    ) {
                                        return true;
                                    }
                                }
                                _ => {}
                            }
                        }
                        false
                    }

                    if let syn::Type::Reference(type_ref_potential) = maybe_ty {
                        let reference_ty =
                            match (type_reference.mutability, type_ref_potential.mutability) {
                                (None, None) => Some(None),
                                (None, Some(_)) => Some(None),
                                //We expect mutable reference, potential_type is not mutable
                                (Some(_), None) => return false,
                                (Some(_), Some(_)) => Some(None),
                            };

                        if handle_path(
                            &type_ref_potential.elem,
                            type_reference,
                            result_args,
                            real_index,
                            maybe_ident,
                            additional_ty,
                            reference_ty,
                        ) {
                            return true;
                        }
                    } else if handle_path(
                        maybe_ty,
                        type_reference,
                        result_args,
                        real_index,
                        maybe_ident,
                        additional_ty,
                        None,
                    ) {
                        return true;
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
            for (real_index, ty) in reference_list.iter() {
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

            let fn_ident = &self.ident;
            let mut result_call_arguments: Vec<Vec<proc_macro2::TokenStream>> = Vec::new();

            let mut list_calls_tokens = quote! {};

            if multiple_calls_allowed {
                let mut result_list_iterators: Vec<proc_macro2::TokenStream> = Vec::new();
                let mut additional_data_pos = 0;
                let mut additional_data_argument = None;

                for (real_pos, potential_args) in result_args_vec.into_iter() {
                    for (index, arg) in potential_args.into_iter().enumerate() {
                        let arg_ident = arg.ident;
                        let mut before_dot = before_dot.clone();
                        let mut clone = quote! {};
                        if let Some(additional_ty) = &arg.additional_ty {
                            before_dot = Default::default();
                            if arg.reference_ty.is_none()
                                && additional_ty == &AdditionalType::NoReference
                            {
                                clone = quote! { .clone() };
                            }
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
                            Some(ReferenceType::Immutable) => {
                                quote! { &#before_dot #arg_ident #clone }
                            }
                            None => quote! { #before_dot #arg_ident #clone },
                        };
                        if arg.additional_ty.is_some() {
                            additional_data_pos = real_pos;
                            additional_data_argument = Some(tokens);
                        } else if let Some(v) = result_call_arguments.get_mut(index) {
                            v.push(tokens);
                        } else {
                            result_call_arguments.push(vec![tokens]);
                        }
                    }
                }

                //Add additional data into every call
                if let Some(additional_data_argument) = additional_data_argument.clone() {
                    for v in result_call_arguments.iter_mut() {
                        v.insert(additional_data_pos, additional_data_argument.clone());
                    }
                }

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

                list_calls_tokens = quote! {
                    #(#list_calls_iter)*
                };
            } else {
                let mut result_single_call_arguments = Vec::new();
                let mut idents_already_used: Vec<syn::Ident> = Vec::new();

                for (_, potential_args) in result_args_vec.iter() {
                    let mut valid_potential_arg_found = false;

                    for arg in potential_args.iter() {
                        if arg.list {
                            //List arguments are not supported if multiple_calls_allowed is false
                            continue;
                        }
                        let arg_ident = arg.ident;
                        if idents_already_used.contains(arg_ident) {
                            continue;
                        } else {
                            idents_already_used.push(arg_ident.clone());
                            valid_potential_arg_found = true;
                        }
                        let mut before_dot = before_dot.clone();
                        let mut clone = quote! {};
                        if let Some(additional_ty) = &arg.additional_ty {
                            before_dot = Default::default();
                            if arg.reference_ty.is_none()
                                && additional_ty == &AdditionalType::NoReference
                            {
                                clone = quote! { .clone() };
                            }
                        }

                        let tokens = match arg.reference_ty {
                            Some(ReferenceType::Mutable) => {
                                quote! { &mut #before_dot #arg_ident #clone }
                            }
                            Some(ReferenceType::Immutable) => {
                                quote! { &#before_dot #arg_ident #clone }
                            }
                            None => quote! { #before_dot #arg_ident #clone },
                        };
                        result_single_call_arguments.push(tokens);
                        break;
                    }

                    if !valid_potential_arg_found {
                        //There were not enough arguments passed in
                        return None;
                    }
                }

                //Check if more than one call can be done (no multiple (potential) calls allowed)
                for (_, potential_args) in result_args_vec.iter() {
                    for arg in potential_args.iter() {
                        if arg.list {
                            //List arguments are not supported if multiple_calls_allowed is false
                            continue;
                        }
                        let arg_ident = arg.ident;
                        if !idents_already_used.contains(arg_ident) {
                            //no multiple (potential) calls allowed
                            return None;
                        }
                    }
                }

                result_call_arguments.push(result_single_call_arguments);
            }

            //Create function calls

            let calls_iter = result_call_arguments.iter().map(|args| {
                quote! {
                    #fn_ident(#(#args),*);
                }
            });

            Some(quote! {
                #(#calls_iter)*
                #list_calls_tokens
            })
        } else {
            None
        }
    }

    pub fn used_check(&self) {
        if !self.used_at_least_once {
            panic!(
                "Function {} was not used while genereting all_syntax_cases macro output",
                self.ident
            );
        }
    }

    pub fn name_equals(&self, name: &syn::Ident) -> bool {
        &self.ident == name
    }

    ///When used outside of search context
    pub fn used(&mut self) {
        self.used_at_least_once = true;
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
    pub option_qself: syn::Ident,
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
        let option_qself = quote::format_ident!("{}_option_qself_handle", fn_name_prefix);

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
            option_qself,

            additional_input_name,
        }
    }
}

pub struct MacroData {
    pub fn_names: MacroFnNames,
    pub additional_input_ty: syn::Type,
    pub default_functions: Vec<EssentialFnData>,
    pub default_functions_after_system: Vec<EssentialFnData>,
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

        //Create function data
        let mut default_functions = Vec::new();
        let mut default_functions_after_system = Vec::new();
        for sig in default_cases.into_iter() {
            if sig.after_system() {
                default_functions_after_system.push(EssentialFnData::new(sig.sig));
            } else {
                default_functions.push(EssentialFnData::new(sig.sig));
            }
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
            option_qself,
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
            fn #option_at_pat(option_at_pat: &mut Option<(Token![@], Box<Pat>)>, #additional_input_name: #additional_input_ty)
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
        system_functions.push(system_new_fn.0(syn::parse_quote! {
            fn #option_qself(option_qself: &mut Option<QSelf>, #additional_input_name: #additional_input_ty)
        }));

        Self {
            fn_names,
            additional_input_ty,
            default_functions,
            default_functions_after_system,
            special_functions,
            system_functions,
        }
    }

    pub fn system_fn_used(&mut self, name_fn: fn(&MacroFnNames) -> &syn::Ident) {
        let name = name_fn(&self.fn_names);

        self.system_functions
            .iter_mut()
            .find(|e| e.name_equals(name))
            .unwrap()
            .used();
    }
}
#[test]
fn type_equals_path_test() {
    let path1: syn::Path = syn::parse_quote!(Path);
    let path2: syn::Path = syn::parse_quote!(syn::Path);
    let path3: syn::Path = syn::parse_quote!(proc_macro2::Path);
    let path4: syn::Path = syn::parse_quote!(Path<syn::V>);
    let path5: syn::Path = syn::parse_quote!(syn::Path<V>);
    let path6: syn::Path = syn::parse_quote!(Path<V>);
    let path7: syn::Path = syn::parse_quote!(Vec<V>);
    let path8: syn::Path = syn::parse_quote!(std::Vec<whatever::lalala::V>);

    //Everything before last segment should be ignored
    assert!(type_equals_path_check(&path1, &path2));
    assert!(type_equals_path_check(&path2, &path3));
    assert!(type_equals_path_check(&path1, &path3));
    //Reverse order check
    assert!(type_equals_path_check(&path2, &path1));
    assert!(type_equals_path_check(&path3, &path2));
    assert!(type_equals_path_check(&path3, &path1));
    //Generics check
    //Shouldn't return true if other type doesn't have generics
    assert!(!type_equals_path_check(&path4, &path1));
    assert!(!type_equals_path_check(&path4, &path2));
    assert!(!type_equals_path_check(&path4, &path3));
    //Everything before last segment should be ignored
    assert!(type_equals_path_check(&path4, &path5));
    assert!(type_equals_path_check(&path4, &path6));
    assert!(type_equals_path_check(&path5, &path6));
    //Reverse order check
    assert!(!type_equals_path_check(&path1, &path4));
    assert!(!type_equals_path_check(&path2, &path4));
    assert!(!type_equals_path_check(&path3, &path4));
    assert!(type_equals_path_check(&path5, &path4));
    assert!(type_equals_path_check(&path6, &path4));
    assert!(type_equals_path_check(&path6, &path5));

    //Not matching last segment
    assert!(!type_equals_path_check(&path7, &path1));
    assert!(!type_equals_path_check(&path7, &path2));
    assert!(!type_equals_path_check(&path7, &path3));
    assert!(!type_equals_path_check(&path7, &path4));
    assert!(!type_equals_path_check(&path7, &path5));
    assert!(!type_equals_path_check(&path7, &path6));
    assert!(!type_equals_path_check(&path8, &path1));
    assert!(!type_equals_path_check(&path8, &path2));
    assert!(!type_equals_path_check(&path8, &path3));
    assert!(!type_equals_path_check(&path8, &path4));
    assert!(!type_equals_path_check(&path8, &path5));
    assert!(!type_equals_path_check(&path8, &path6));
    //Reverse order check
    assert!(!type_equals_path_check(&path1, &path7));
    assert!(!type_equals_path_check(&path2, &path7));
    assert!(!type_equals_path_check(&path3, &path7));
    assert!(!type_equals_path_check(&path4, &path7));
    assert!(!type_equals_path_check(&path5, &path7));
    assert!(!type_equals_path_check(&path6, &path7));
    assert!(!type_equals_path_check(&path1, &path8));
    assert!(!type_equals_path_check(&path2, &path8));
    assert!(!type_equals_path_check(&path3, &path8));
    assert!(!type_equals_path_check(&path4, &path8));
    assert!(!type_equals_path_check(&path5, &path8));
    assert!(!type_equals_path_check(&path6, &path8));

    //Matching generics (last segment) and last segment
    assert!(type_equals_path_check(&path7, &path8));
    //Reverse order check
    assert!(type_equals_path_check(&path8, &path7));
}

#[cfg(test)]
fn test_vec_eq(vec1: &[syn::Type], vec2: &[syn::Type], expected: bool) {
    for item1 in vec1.iter() {
        for item2 in vec2.iter() {
            assert_eq!(
                type_equals(item1, item2),
                expected,
                "type_equals({}, {}) should be {}",
                item1.to_token_stream(),
                item2.to_token_stream(),
                expected
            );
        }
    }
}

#[test]
fn type_equals_test() {
    let mut all_vectors = Vec::new();

    let vec_item: Vec<syn::Type> = vec![
        syn::parse_quote!(Vec<syn::Item>),
        syn::parse_quote!(Vec<Item>),
        syn::parse_quote!(std::Vec<Item>),
    ];
    test_vec_eq(&vec_item, &vec_item, true);
    all_vectors.push(vec_item);

    let expr: Vec<syn::Type> = vec![syn::parse_quote!(Expr), syn::parse_quote!(syn::Expr)];
    test_vec_eq(&expr, &expr, true);
    all_vectors.push(expr);

    let option_expr: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<Expr>),
        syn::parse_quote!(Option<syn::Expr>),
    ];
    test_vec_eq(&option_expr, &option_expr, true);
    all_vectors.push(option_expr);

    let where_predicate: Vec<syn::Type> = vec![
        syn::parse_quote!(WherePredicate),
        syn::parse_quote!(syn::WherePredicate),
    ];
    test_vec_eq(&where_predicate, &where_predicate, true);
    all_vectors.push(where_predicate);

    let where_clause: Vec<syn::Type> = vec![
        syn::parse_quote!(WhereClause),
        syn::parse_quote!(syn::WhereClause),
    ];
    test_vec_eq(&where_clause, &where_clause, true);
    all_vectors.push(where_clause);

    let option_pat: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<(Box<Pat>, Token![:])>),
        syn::parse_quote!(Option<(Box<syn::Pat>, Token![:])>),
        syn::parse_quote!(Option<(Box<Pat>, syn::Token![:])>),
    ];
    test_vec_eq(&option_pat, &option_pat, true);
    all_vectors.push(option_pat);

    let variadic: Vec<syn::Type> = vec![
        syn::parse_quote!(Variadic),
        syn::parse_quote!(syn::Variadic),
    ];
    test_vec_eq(&variadic, &variadic, true);
    all_vectors.push(variadic);

    let option_variadic: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<Variadic>),
        syn::parse_quote!(Option<syn::Variadic>),
    ];
    test_vec_eq(&option_variadic, &option_variadic, true);
    all_vectors.push(option_variadic);

    let option_fields: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<(token::Brace, Vec<Item>)>),
        syn::parse_quote!(Option<(syn::token::Brace, Vec<Item>)>),
        syn::parse_quote!(Option<(Brace, Vec<syn::Item>)>),
    ];
    test_vec_eq(&option_fields, &option_fields, true);
    all_vectors.push(option_fields);

    let option_fields_reversed: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<(Vec<Item>, token::Brace)>),
        syn::parse_quote!(Option<(Vec<Item>, syn::token::Brace)>),
        syn::parse_quote!(Option<(Vec<syn::Item>, Brace)>),
        syn::parse_quote!(Option<(Vec<syn::Item>, token::Brace)>),
        syn::parse_quote!(Option<(Vec<syn::Item>, syn::token::Brace)>),
    ];
    test_vec_eq(&option_fields_reversed, &option_fields_reversed, true);
    all_vectors.push(option_fields_reversed);

    let fields: Vec<syn::Type> = vec![syn::parse_quote!(Fields), syn::parse_quote!(syn::Fields)];
    test_vec_eq(&fields, &fields, true);
    all_vectors.push(fields);

    let field: Vec<syn::Type> = vec![syn::parse_quote!(Field), syn::parse_quote!(syn::Field)];
    test_vec_eq(&field, &field, true);
    all_vectors.push(field);

    let field_pat: Vec<syn::Type> = vec![
        syn::parse_quote!(FieldPat),
        syn::parse_quote!(syn::FieldPat),
    ];
    test_vec_eq(&field_pat, &field_pat, true);
    all_vectors.push(field_pat);

    let option_expr_eq: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<(Token![=], Expr)>),
        syn::parse_quote!(Option<(syn::Token![=], Expr)>),
        syn::parse_quote!(Option<(Token![=], syn::Expr)>),
    ];
    test_vec_eq(&option_expr_eq, &option_expr_eq, true);
    all_vectors.push(option_expr_eq);

    let option_box_expr: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<Box<Expr>>),
        syn::parse_quote!(Option<Box<syn::Expr>>),
    ];
    test_vec_eq(&option_box_expr, &option_box_expr, true);
    all_vectors.push(option_box_expr);

    let option_at_pat: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<(Token![@], Box<Pat>)>),
        syn::parse_quote!(Option<(Token![@], Box<syn::Pat>)>),
        syn::parse_quote!(Option<(syn::Token![@], Box<Pat>)>),
        syn::parse_quote!(Option<(syn::Token![@], Box<syn::Pat>)>),
    ];
    test_vec_eq(&option_at_pat, &option_at_pat, true);
    all_vectors.push(option_at_pat);

    let option_at_pat_fake: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<(Token![!], Box<Pat>)>),
        syn::parse_quote!(Option<(Token![!], Box<syn::Pat>)>),
        syn::parse_quote!(Option<(syn::Token![!], Box<Pat>)>),
        syn::parse_quote!(Option<(syn::Token![!], Box<syn::Pat>)>),
    ];
    test_vec_eq(&option_at_pat_fake, &option_at_pat_fake, true);
    all_vectors.push(option_at_pat_fake);

    let option_else_expr: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<(Token![else], Box<Expr>)>),
        syn::parse_quote!(Option<(syn::Token![else], Box<Expr>)>),
        syn::parse_quote!(Option<(Token![else], Box<syn::Expr>)>),
        syn::parse_quote!(Option<(syn::Token![else], Box<syn::Expr>)>),
    ];
    test_vec_eq(&option_else_expr, &option_else_expr, true);
    all_vectors.push(option_else_expr);

    let option_if_expr: Vec<syn::Type> = vec![
        syn::parse_quote!(Option<(Token![if], Box<Expr>)>),
        syn::parse_quote!(Option<(syn::Token![if], Box<Expr>)>),
        syn::parse_quote!(Option<(Token![if], Box<syn::Expr>)>),
        syn::parse_quote!(Option<(syn::Token![if], Box<syn::Expr>)>),
    ];
    test_vec_eq(&option_if_expr, &option_if_expr, true);
    all_vectors.push(option_if_expr);

    //Different Vectors (representing different equal values) should never have equal values (between different vectors)
    for v in all_vectors.iter() {
        for v2 in all_vectors.iter() {
            if v == v2 {
                continue;
            }
            test_vec_eq(v, v2, false);
        }
    }
}

#[test]
fn essential_fn_checks_1_arg_test() {
    // struct AdditionalInput;

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

    let input_fields2 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: syn::Item,
                b: syn::Expr,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
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
            .all_inputs_check(&input_fields3, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x);
            }
            .to_string()
        )
    );

    let input_fields4 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                b: Box<syn::Expr>,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields4, None, additional_input)
            .map(|x| x.to_string()),
        None
    );

    let input_fields5 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: Box<syn::Item>,
                b: Box<syn::Expr>,
                c: Box<syn::Item>,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields5, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x);
                example_fn1(&mut c);
            }
            .to_string()
        )
    );

    let input_fields6 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: Box<syn::Item>,
                b: Box<syn::Expr>,
                list: Vec<syn::Item>,
                c: Box<syn::Item>,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields6, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x);
                example_fn1(&mut c);
                for ____x in list.iter_mut(){
                    example_fn1(____x);
                }
            }
            .to_string()
        )
    );

    let input_fields7 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: Box<syn::Item>,
                list0: syn::Punctuated<syn::Item, Token![,]>,
                b: Box<syn::Expr>,
                list: Vec<syn::Item>,
                c: Box<syn::Item>,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields7, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x);
                example_fn1(&mut c);
                for ____x in list0.iter_mut(){
                    example_fn1(____x);
                }
                for ____x in list.iter_mut(){
                    example_fn1(____x);
                }
            }
            .to_string()
        )
    );
}

#[test]
fn essential_fn_checks_2_args_test() {
    // struct AdditionalInput;

    let additional_input_base: (syn::Ident, syn::Type) = (
        quote::format_ident!("__additional_input"),
        syn::parse_quote! {AdditionalInput},
    );
    let additional_input = (&additional_input_base.0, &additional_input_base.1);

    let mut fn_data1 = EssentialFnData::new(syn::parse_quote! {
        fn example_fn1(a: &mut syn::Expr, __additional_input: &mut AdditionalInput)
    });
    let mut fn_data2 = EssentialFnData::new(syn::parse_quote! {
        fn example_fn2(a: &mut syn::Expr, __additional_input: AdditionalInput)
    });

    let input_fields1 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: &mut syn::Expr,
                b: &mut syn::Stmt,
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
                example_fn1(x, &mut __additional_input);
            }
            .to_string()
        )
    );
    assert_eq!(
        fn_data2
            .all_inputs_check(&input_fields1, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn2(x, __additional_input.clone());
            }
            .to_string()
        )
    );

    let input_fields2 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: syn::Expr,
                b: syn::Stmt,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields2, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x, &mut __additional_input);
            }
            .to_string()
        )
    );
    assert_eq!(
        fn_data2
            .all_inputs_check(&input_fields2, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn2(&mut x, __additional_input.clone());
            }
            .to_string()
        )
    );

    let input_fields3 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: Box<syn::Expr>,
                b: Box<syn::Stmt>,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields3, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x, &mut __additional_input);
            }
            .to_string()
        )
    );
    assert_eq!(
        fn_data2
            .all_inputs_check(&input_fields3, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn2(&mut x, __additional_input.clone());
            }
            .to_string()
        )
    );

    let input_fields4 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                b: Box<syn::Stmt>,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields4, None, additional_input)
            .map(|x| x.to_string()),
        None
    );
    assert_eq!(
        fn_data2
            .all_inputs_check(&input_fields4, None, additional_input)
            .map(|x| x.to_string()),
        None
    );

    let input_fields5 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: Box<syn::Expr>,
                b: Box<syn::Stmt>,
                c: Box<syn::Expr>,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields5, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x, &mut __additional_input);
                example_fn1(&mut c, &mut __additional_input);
            }
            .to_string()
        )
    );
    assert_eq!(
        fn_data2
            .all_inputs_check(&input_fields5, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn2(&mut x, __additional_input.clone());
                example_fn2(&mut c, __additional_input.clone());
            }
            .to_string()
        )
    );

    let input_fields6 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: Box<syn::Expr>,
                b: Box<syn::Stmt>,
                list: Vec<syn::Expr>,
                c: Box<syn::Expr>,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields6, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x, &mut __additional_input);
                example_fn1(&mut c, &mut __additional_input);
                for ____x in list.iter_mut(){
                    example_fn1(____x, &mut __additional_input);
                }
            }
            .to_string()
        )
    );
    assert_eq!(
        fn_data2
            .all_inputs_check(&input_fields6, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn2(&mut x, __additional_input.clone());
                example_fn2(&mut c, __additional_input.clone());
                for ____x in list.iter_mut(){
                    example_fn2(____x, __additional_input.clone());
                }
            }
            .to_string()
        )
    );
    let input_fields7 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: Box<syn::Expr>,
                list0: syn::Punctuated<syn::Expr, Token![,]>,
                b: Box<syn::Stmt>,
                list: Vec<syn::Expr>,
                c: Box<syn::Expr>,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields7, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn1(&mut x, &mut __additional_input);
                example_fn1(&mut c, &mut __additional_input);
                for ____x in list0.iter_mut(){
                    example_fn1(____x, &mut __additional_input);
                }
                for ____x in list.iter_mut(){
                    example_fn1(____x, &mut __additional_input);
                }
            }
            .to_string()
        )
    );
    assert_eq!(
        fn_data2
            .all_inputs_check(&input_fields7, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn2(&mut x, __additional_input.clone());
                example_fn2(&mut c, __additional_input.clone());
                for ____x in list0.iter_mut(){
                    example_fn2(____x, __additional_input.clone());
                }
                for ____x in list.iter_mut(){
                    example_fn2(____x, __additional_input.clone());
                }
            }
            .to_string()
        )
    );
}

#[test]
fn essential_fn_checks_3_args_test() {
    // struct AdditionalInput;

    let additional_input_base: (syn::Ident, syn::Type) = (
        quote::format_ident!("__additional_input"),
        syn::parse_quote! {AdditionalInput},
    );
    let additional_input = (&additional_input_base.0, &additional_input_base.1);

    let mut fn_data1 = EssentialFnData::new(syn::parse_quote! {
        fn example_fn1(a: &mut syn::Expr, b: &mut syn::Item, __additional_input: &mut AdditionalInput)
    });
    let mut fn_data2 = EssentialFnData::new(syn::parse_quote! {
        fn example_fn2(a: &mut syn::Expr, b: &mut syn::Expr, __additional_input: &mut AdditionalInput)
    });

    let input_fields1 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: &mut syn::Expr,
                b: &mut syn::Item,
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
                example_fn1(x, b, &mut __additional_input);
            }
            .to_string()
        )
    );
    assert_eq!(
        fn_data2
            .all_inputs_check(&input_fields1, None, additional_input)
            .map(|x| x.to_string()),
        None
    );

    let input_fields2 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: syn::Expr,
                b: syn::Expr,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields2, None, additional_input)
            .map(|x| x.to_string()),
        None
    );
    assert_eq!(
        fn_data2
            .all_inputs_check(&input_fields2, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn2(&mut x, &mut b, &mut __additional_input);
            }
            .to_string()
        )
    );

    let input_fields3 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                x: syn::Expr,
                b: syn::Expr,
                a: syn::Item,
                a2: syn::Item,
            }
        };

        input_fields.named.into_iter().collect::<Vec<_>>()
    };
    assert_eq!(
        fn_data1
            .all_inputs_check(&input_fields3, None, additional_input)
            .map(|x| x.to_string()),
        None
    );
    assert_eq!(
        fn_data2
            .all_inputs_check(&input_fields3, None, additional_input)
            .map(|x| x.to_string()),
        Some(
            quote! {
                example_fn2(&mut x, &mut b, &mut __additional_input);
            }
            .to_string()
        ),
    );
}

#[test]
fn essential_fn_checks_return_type_debug_test() {
    let additional_input_base: (syn::Ident, syn::Type) = (
        quote::format_ident!("__additional_input"),
        syn::parse_quote! {AdditionalInput},
    );
    let additional_input = (&additional_input_base.0, &additional_input_base.1);

    //Debug issues with ReturnType (because of &mut Box<Type>)
    let mut fn_data1 = EssentialFnData::new(syn::parse_quote! {
        fn example_ty_handle(ty: &mut Type, __additional_input: AdditionalInput)
    });

    let input_fields1 = {
        let input_fields: syn::FieldsNamed = syn::parse_quote! {
            {
                arg1: &mut Token![->], arg2: &mut Box<Type>
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
                example_ty_handle(arg2, __additional_input.clone());
            }
            .to_string()
        )
    );
}
