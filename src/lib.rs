#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use proc_macro::{Diagnostic, Level::*};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, ItemFn, Signature};

#[proc_macro_attribute]
pub fn auto_curry(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(input as ItemFn);

    if attrs.len() > 0 {
        Diagnostic::spanned(
            attrs
                .into_iter()
                .map(|attr| attr.span().unwrap())
                .collect::<Vec<_>>(),
            Warning,
            "Other attributes on curried functions are unsupported, ignoring this",
        )
        .emit();
    }

    let Signature {
        constness,
        asyncness,
        unsafety,
        abi,
        ident,
        generics,
        inputs,
        output,
        ..
    } = sig;
    

    let output = quote! { #vis #constness#asyncness#unsafety #abi #ident #generics #inputs #output #block };
    output.into()
}
