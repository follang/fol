#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use proc_macro::{Diagnostic, TokenStream};
use syn::{parse2, DeriveInput};

mod enumeq;

type DeriveFn = fn(DeriveInput) -> Result<proc_macro2::TokenStream, Diagnostic>;

#[proc_macro_derive(EnumEq)]
pub fn varianteq_derive(tokens: TokenStream) -> TokenStream {
    expand_derive(tokens, enumeq::derive)
}

fn expand_derive(tokens: TokenStream, derive: DeriveFn) -> TokenStream {
    let item = parse2(tokens.into()).unwrap();
    match derive(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => handle_derive_err(err),
    }
}

fn handle_derive_err(err: Diagnostic) -> TokenStream {
    err.emit();
    "".parse().unwrap()
}
