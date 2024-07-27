//! Derives for `LibAFL`

#![no_std]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro to implement `SerdeAny`, to use a type in a `SerdeAnyMap`
#[proc_macro_derive(SerdeAny)]
pub fn libafl_serdeany_derive(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as DeriveInput).ident;
    TokenStream::from(quote! {
        libafl_bolts::impl_serdeany!(#name);
    })
}
