use quote::{ToTokens, quote};
use syn::{FieldValue, Signature, Token, punctuated::Punctuated};

pub struct InputSetup {
    starting_point_type: syn::Path,
    generated_fn_prefix: String,
    additional_input_type: syn::Path,
}

impl syn::parse::Parse for InputSetup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parsed: Punctuated<FieldValue, Token![,]> = Punctuated::parse_terminated(&input)?;

        let mut starting_point_type = None;
        let mut generated_fn_prefix = None;
        let mut additional_input_type = None;

        for el in parsed.into_iter() {
            match el.member {
                syn::Member::Named(ident) => {
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "starting_point_type" => {
                            if let syn::Expr::Path(expr_path) = el.expr {
                                starting_point_type = Some(expr_path.path);
                            } else {
                                panic!("starting_point_type must be a path");
                            }
                        }
                        "generated_fn_prefix" => {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = el.expr
                            {
                                generated_fn_prefix = Some(lit_str.value());
                            } else {
                                panic!("generated_fn_prefix must be a string");
                            }
                        }
                        "additional_input_type" => {
                            if let syn::Expr::Path(expr_path) = el.expr {
                                additional_input_type = Some(expr_path.path);
                            } else {
                                panic!("additional_input_type must be a path");
                            }
                        }
                        _ => {}
                    }
                }
                syn::Member::Unnamed(_) => panic!("unnamed member not supported"),
            }
        }

        Ok(InputSetup {
            starting_point_type: starting_point_type
                .expect("starting_point_type was not provided inside of setup => {...}"),
            generated_fn_prefix: generated_fn_prefix
                .expect("generated_fn_prefix was not provided inside of setup => {...}"),
            additional_input_type: additional_input_type
                .expect("additional_input_type was not provided inside of setup => {...}"),
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

pub struct EssentialFnData {
    input_types: Vec<syn::Type>,
    ident: syn::Ident,
    ///Used for showing errors (if false)
    used_at_least_once: bool,
}

pub enum ReferenceType {
    Mutable,
    Immutable,
    // ///Aka Dereference
    // Unbox,
}

impl EssentialFnData {
    pub fn new(sig: Signature) -> Self {
        let mut input_types = Vec::new();

        //TODO Handle generics Someday
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

    ///Returns function call if all inputs are present
    pub fn all_inputs_check(
        &mut self,
        fields: &[syn::Field],
        before_dot: Option<&proc_macro2::TokenStream>,
        additional_input: (&syn::Ident, &syn::Path),
    ) -> Option<proc_macro2::TokenStream> {
        let (additional_input_ident, additional_input_ty) = additional_input;
        let additional_input_ty_str = additional_input_ty.to_token_stream().to_string();

        //Create reference list from required input types
        let mut reference_list = self.input_types.iter().enumerate().collect::<Vec<_>>();

        //Type used for creating final arguments list
        struct ResultArgData<'a> {
            real_index: usize,
            ident: &'a syn::Ident,
            reference_ty: Option<ReferenceType>,
        }

        let mut result_args = Vec::new();

        //Remove additional input type from reference list
        for (index, (real_index, ty)) in reference_list.iter().enumerate() {
            if ty.to_token_stream().to_string() == additional_input_ty_str {
                result_args.push(ResultArgData {
                    real_index: *real_index,
                    ident: additional_input_ident,
                    reference_ty: None,
                });
                reference_list.remove(index);
                break;
            } else {
                if let syn::Type::Reference(type_reference) = ty {
                    if &*type_reference.elem.to_token_stream().to_string()
                        == additional_input_ty_str
                    {
                        let reference_ty = if type_reference.mutability.is_some() {
                            Some(ReferenceType::Mutable)
                        } else {
                            Some(ReferenceType::Immutable)
                        };

                        result_args.push(ResultArgData {
                            real_index: *real_index,
                            ident: additional_input_ident,
                            reference_ty,
                        });

                        reference_list.remove(index);
                        break;
                    }
                }
            }
        }

        //Remove all found types from reference list
        for field in fields.iter() {
            for (index, (real_index, ty)) in reference_list.iter().enumerate() {
                if ty == &&field.ty {
                    result_args.push(ResultArgData {
                        real_index: *real_index,
                        ident: field.ident.as_ref().unwrap(),
                        reference_ty: None,
                    });

                    reference_list.remove(index);
                    break;
                } else {
                    if let syn::Type::Reference(type_reference) = ty {
                        if *type_reference.elem == field.ty {
                            let reference_ty = if type_reference.mutability.is_some() {
                                Some(ReferenceType::Mutable)
                            } else {
                                Some(ReferenceType::Immutable)
                            };

                            result_args.push(ResultArgData {
                                real_index: *real_index,
                                ident: field.ident.as_ref().unwrap(),
                                reference_ty,
                            });

                            reference_list.remove(index);
                            break;
                        } else if let syn::Type::Path(ty_path) = &field.ty {
                            //Handle Boxes
                            if let Some(ident) = ty_path.path.get_ident() {
                                if ident.to_string().as_str() == "Box" {
                                    //Get Type inside of <>
                                    if let Some(qself) = &ty_path.qself {
                                        if type_reference.elem == qself.ty {
                                            let reference_ty =
                                                if type_reference.mutability.is_some() {
                                                    Some(ReferenceType::Mutable)
                                                } else {
                                                    Some(ReferenceType::Immutable)
                                                };

                                            result_args.push(ResultArgData {
                                                real_index: *real_index,
                                                ident: field.ident.as_ref().unwrap(),
                                                reference_ty,
                                            });

                                            reference_list.remove(index);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if reference_list.is_empty() {
            self.used_at_least_once = true;

            //Sort arguments
            result_args.sort_by(|a, b| a.real_index.cmp(&b.real_index));
            //Format arguments
            let before_dot = if let Some(before_dot) = before_dot {
                quote! {#before_dot.}
            } else {
                quote! {}
            };
            let args_for_quote = result_args.iter().map(|arg| {
                let arg_ident = arg.ident;
                match arg.reference_ty {
                    Some(ReferenceType::Mutable) => quote! { &mut #before_dot #arg_ident },
                    Some(ReferenceType::Immutable) => quote! { &#before_dot #arg_ident },
                    None => quote! { #arg_ident },
                }
            });
            //Create function call
            let fn_ident = &self.ident;
            Some(quote! {
                #fn_ident(#(#args_for_quote),*);
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

            additional_input_name,
        }
    }
}

pub struct MacroData {
    pub fn_names: MacroFnNames,
    pub generated_fn_prefix: String,
    pub additional_input_ty: syn::Path,
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
        } = &fn_names;
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #item(item: &mut syn::Item, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #expr(expr: &mut syn::Expr, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #expr_option(expr_option: &mut Option<syn::Expr>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #block(block: &mut syn::Block, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #stmt(stmt: &mut syn::Stmt, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #generic_param(generic_param: &mut syn::GenericParam, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #generics(generics: &mut syn::Generics, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #type_param_bound(type_param_bound: &mut syn::TypeParamBound, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #bound_lifetimes(bound_lifetimes: &mut syn::BoundLifetimes, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #bound_lifetimes_option(bound_lifetimes_option: &mut Option<syn::BoundLifetimes>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #where_predicate(where_predicate: &mut syn::WherePredicate, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #impl_item(impl_item: &mut syn::ImplItem, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #item_mod_content(item_mod_content: &mut syn::ItemModContent, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #fields(fields: &mut syn::Fields, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #trait_item(trait_item: &mut syn::TraitItem, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #fields_named(fields_named: &mut syn::FieldsNamed, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #option_box_expr(option_box_expr: &mut Option<syn::ExprBox>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #pat(pat: &mut syn::Pat, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #option_else_expr(option_else_expr: &mut Option<syn::Expr>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #arm(arm: &mut syn::Arm, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #angle_bracketed_generic_arguments(angle_bracketed_generic_arguments: &mut syn::AngleBracketedGenericArguments, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #range_limits(range_limits: &mut syn::RangeLimits, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #field_value(field_value: &mut syn::FieldValue, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #local_init(local_init: &mut syn::Local, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #option_local_init(option_local_init: &mut Option<syn::Local>, #additional_input_name: #additional_input_ty)
        }));
        system_functions.push(EssentialFnData::new(syn::parse_quote! {
            fn #signature(signature: &mut syn::Signature, #additional_input_name: #additional_input_ty)
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
