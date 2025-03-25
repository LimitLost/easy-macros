use always_context::always_context;

pub struct HandleAttrsInput {
    pub operate_on: syn::Expr,
    _comma: syn::token::Comma,
    pub attributes: Vec<syn::Attribute>,
}

#[always_context]
impl syn::parse::Parse for HandleAttrsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let operate_on = input.parse()?;
        let _comma = input.parse()?;
        let attributes = syn::Attribute::parse_outer(input)?;

        Ok(HandleAttrsInput {
            operate_on,
            _comma,
            attributes,
        })
    }
}

pub enum Reference {
    Ref,
    RefMut,
}

pub struct HandleMaybeRefAttrsInput {
    pub reference: Option<Reference>,
    pub operate_on: syn::Expr,
    _comma: syn::token::Comma,
    pub attributes: Vec<syn::Attribute>,
}

#[always_context]
impl syn::parse::Parse for HandleMaybeRefAttrsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let reference = if input.peek(syn::Token![&]) {
            let _: syn::Token![&] = input.parse()?;
            if input.peek(syn::Token![mut]) {
                let _: syn::Token![mut] = input.parse()?;
                Some(Reference::RefMut)
            } else {
                Some(Reference::Ref)
            }
        } else {
            None
        };
        let operate_on = input.parse()?;
        let _comma = input.parse()?;
        let attributes = syn::Attribute::parse_outer(input)?;

        Ok(HandleMaybeRefAttrsInput {
            reference,
            operate_on,
            _comma,
            attributes,
        })
    }
}
