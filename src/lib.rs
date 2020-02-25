#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use proc_macro::{Diagnostic, Level::*};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, ItemFn, Signature};

struct CurryStep<'a> {
    ident: syn::Ident,
    args: &'a [syn::Type],
    rest: &'a [syn::Type],
}

impl<'a> ToTokens for CurryStep<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let CurryStep { ident, args, .. } = self;
        let args = args.iter();
        tokens.extend(
            quote! {
                #[allow(nonstandard_style)] struct #ident(#(#args),*);
            }
            .into_iter(),
        );
    }
}

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
        unsafety: _,
        abi: _,
        ident,
        generics: _,
        inputs: fn_inputs,
        output,
        ..
    } = sig;
    let inputs = match fn_inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(ty) => Ok(*ty.ty.clone()),
            syn::FnArg::Receiver(slf) => Err(syn::Error::new_spanned(
                slf,
                "Looks like you're trying to curry member function. This is not supported yet",
            )),
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(v) => v,
        Err(er) => return er.to_compile_error().into(),
    };

    if let Some(constness) = constness {
        Diagnostic::spanned(
            constness.span().unwrap(),
            Warning,
            "Curried functions can't be const, ignoring",
        )
        .emit();
    }
    if let Some(asyncness) = asyncness {
        return syn::Error::new_spanned(
            asyncness,
            "It's impossible yet to make async function be curried",
        )
        .to_compile_error()
        .into();
    }

    let base_type = format_ident!("{}_BASE_CURRIED", ident);
    let base = quote! {
        #[allow(nonstandard_style)] #vis const #ident: #base_type = #base_type();
    };

    let mut steps = Vec::with_capacity(inputs.len());

    let (args, rest) = inputs.split_at(0);
    steps.push(CurryStep {
        ident: base_type.clone(),
        args,
        rest,
    });

    for i in 1..inputs.len() {
        let (args, rest) = inputs.split_at(i);
        steps.push(CurryStep {
            ident: format_ident!("{}_CURRIED_STEP_{}", ident, i),
            args,
            rest,
        })
    }

    let base_impls = steps.iter().skip(1).map(|CurryStep { ident, args, .. }| {
        let unpacked = args.iter().enumerate().map(|(index, _)| {
            let index = syn::Index::from(index);
            quote! {args.#index}
        });
        quote! {
            impl FnOnce<(#(#args),*,)> for #base_type {
                type Output = #ident;
                extern "rust-call" fn call_once(self, args: (#(#args),*,)) -> #ident {
                    #ident(#(#unpacked),*)
                }
            }
        }
    });

    let steps_count = steps.len();
    let mut intermediate_impls = Vec::with_capacity((steps_count - 2) * (steps_count - 1) / 2);
    for from in 1..steps_count {
        for to in from + 1..steps_count {
            let type_from = &steps[from].ident;
            let type_to = &steps[to].ident;
            let args = &steps[from].rest[0..to - from];
            let unpacked = steps[from]
                .args
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, _)| {
                    let index = syn::Index::from(index);
                    quote! {self.#index}
                })
                .chain(args.iter().cloned().enumerate().map(|(index, _)| {
                    let index = syn::Index::from(index);
                    quote! {args.#index}
                }));
            intermediate_impls.push(quote! {
                impl FnOnce<(#(#args),*,)> for #type_from {
                    type Output = #type_to;
                    extern "rust-call" fn call_once(self, args: (#(#args),*,)) -> #type_to {
                        #type_to(#(#unpacked),*)
                    }
                }
            });
        }
    }

    let real_impl_name = format_ident!("{}_REAL_IMPL", ident);
    let real_impl =
        quote! { #[allow(nonstandard_style)] fn #real_impl_name(#fn_inputs) #output #block };

    let final_impls = steps.iter().map(|CurryStep { ident, args, rest }| {
        let unpacked_args = args.iter().enumerate().map(|(index, _)| {
            let index = syn::Index::from(index);
            quote! {self.#index}
        });
        let unpacked_rest = rest.iter().enumerate().map(|(index, _)| {
            let index = syn::Index::from(index);
            quote! {args.#index}
        });
        let unpacked = unpacked_args.chain(unpacked_rest);
        let output = match &output {
            syn::ReturnType::Default => quote! { () },
            syn::ReturnType::Type(_, ty) => ty.to_token_stream().clone(),
        };
        quote! {
            impl FnOnce<(#(#rest),*,)> for #ident {
                type Output = #output;
                extern "rust-call" fn call_once(self, args: (#(#rest),*,)) -> #output {
                    #real_impl_name(#(#unpacked),*)
                }
            }
        }
    });

    let output = quote! {
        #base
        #(#steps)*
        #(#base_impls)*
        #(#intermediate_impls)*
        #(#final_impls)*
        #real_impl
    };
    output.into()
}
