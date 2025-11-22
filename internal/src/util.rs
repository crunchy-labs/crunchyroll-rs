use darling::FromMeta;
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Ident, Meta};

pub struct IdentList {
    span: Span,
    idents: Punctuated<Ident, Comma>,
}

impl Parse for IdentList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punctuated = Punctuated::<Ident, Comma>::parse_terminated(input)?;
        Ok(Self {
            span: input.span(),
            idents: punctuated,
        })
    }
}

impl FromMeta for IdentList {
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        syn::parse2::<IdentList>(item.require_list()?.tokens.clone())
            .map_err(|e| darling::Error::custom(e.to_string()))
    }
}

impl IdentList {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn to_vec(&self) -> Vec<Ident> {
        self.idents.iter().cloned().collect()
    }
}
