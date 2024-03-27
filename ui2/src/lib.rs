use proc_macro2::TokenStream;
use syn::{parse::Parse, Ident};


pub struct Widget {
    name: Ident,
    props: Props
}

pub struct Props {

}

pub enum Prop {
    
}

impl Parse for Props {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

impl Parse for Widget {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let content;
        syn::bracketed!(content in input);
        let props = content.parse()?;

        Ok(Widget {
            name,
            props
        })
    }
}

pub fn parse(stream: TokenStream) -> TokenStream {
    todo!()
}