use quote::{ToTokens, quote};
use syn::{FieldValue, Signature, Token, punctuated::Punctuated};

pub struct InputSetup {
    entry_ty: syn::Path,
    fn_name_prefix: String,
    additional_input_ty: syn::Path,
}

impl syn::parse::Parse for InputSetup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parsed: Punctuated<FieldValue, Token![,]> = Punctuated::parse_terminated(&input)?;

        let mut entry_ty = None;
        let mut fn_name_prefix = None;
        let mut additional_input_ty = None;

        for el in parsed.into_iter() {
            match el.member {
                syn::Member::Named(ident) => {
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "entry_ty" => {
                            if let syn::Expr::Path(expr_path) = el.expr {
                                entry_ty = Some(expr_path.path);
                            } else {
                                panic!("entry_ty must be a path");
                            }
                        }
                        "fn_name_prefix" => {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = el.expr
                            {
                                fn_name_prefix = Some(lit_str.value());
                            } else {
                                panic!("fn_name_prefix must be a string");
                            }
                        }
                        "additional_input_ty" => {
                            if let syn::Expr::Path(expr_path) = el.expr {
                                additional_input_ty = Some(expr_path.path);
                            } else {
                                panic!("additional_input_ty must be a path");
                            }
                        }
                        _ => {}
                    }
                }
                syn::Member::Unnamed(_) => panic!("unnamed member not supported"),
            }
        }

        Ok(InputSetup {
            entry_ty: entry_ty.expect("entry_ty was not provided inside of setup=>{...}"),
            fn_name_prefix: fn_name_prefix
                .expect("fn_name_prefix was not provided inside of setup=>{...}"),
            additional_input_ty: additional_input_ty
                .expect("additional_input_ty was not provided inside of setup=>{...}"),
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

        let setup = setup.expect("setup was not provided! Usage: setup => { <entry_ty, fn_name_prefix, additional_input_ty> }");
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
    //TODO Item etc.
}

pub struct MacroData {
    pub fn_names: MacroFnNames,
    pub fn_name_prefix: String,
    pub additional_input_ty: syn::Path,
    pub default_functions: Vec<EssentialFnData>,
    pub special_functions: Vec<EssentialFnData>,
}
