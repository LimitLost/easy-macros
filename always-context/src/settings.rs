use syn::parse::Parse;

/// Expected format for the `skip(...)` options: `skip(macros, easy_sql, ...)` or shorthand `skip(!, es, ...)`
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// Expected input format: `skip(macros)` or (shorthand) `skip(!)`
    pub skip_macros: bool,
    /// Expected input format: `skip(easy_sql)` or (shorthand) `skip(es)`
    #[cfg(feature = "easy-sql")]
    pub easy_sql: bool,
}

mod kw {
    syn::custom_keyword!(skip);
}

impl Parse for Settings {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let mut skip_macros = false;
        #[cfg(feature = "easy-sql")]
        let mut easy_sql = false;
        if lookahead.peek(kw::skip) {
            input.parse::<kw::skip>()?;
            let content;
            syn::parenthesized!(content in input);

            while !content.is_empty() {
                let lookahead = content.lookahead1();
                if lookahead.peek(syn::Token![!]) {
                    let _bang: syn::Token![!] = content.parse()?;
                    skip_macros = true;
                } else if lookahead.peek(syn::Ident) {
                    let ident: syn::Ident = content.parse()?;
                    match ident.to_string().as_str() {
                        "macros" => skip_macros = true,
                        #[cfg(feature = "easy-sql")]
                        "easy_sql" | "es" => easy_sql = true,
                        _ => {
                            return Err(syn::Error::new(
                                ident.span(),
                                format!("Unknown option: {ident}"),
                            ));
                        }
                    }
                } else {
                    return Err(lookahead.error());
                }

                if content.peek(syn::Token![,]) {
                    let _comma: syn::Token![,] = content.parse()?;
                }
            }
        }

        Ok(Settings {
            skip_macros,
            #[cfg(feature = "easy-sql")]
            easy_sql,
        })
    }
}
