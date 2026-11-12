use proc_macro2::TokenStream;
#[cfg(test)]
use quote::quote;
use syn::{Expr, ItemFn, Lit, MetaNameValue};

pub fn expand(attr: TokenStream, body: ItemFn) -> syn::Result<TokenStream> {
    let seeds = if attr.is_empty() {
        100u64
    } else {
        let argument: MetaNameValue = syn::parse2(attr)?;
        match &argument.value {
            Expr::Lit(value) if argument.path.is_ident("seeds") => match &value.lit {
                Lit::Int(value) => value.base10_parse::<u64>()?,
                _ => {
                    return Err(syn::Error::new_spanned(
                        argument,
                        "expected `seeds = N` with a u64 literal",
                    ));
                }
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    argument,
                    "expected `seeds = N` with a u64 literal",
                ));
            }
        }
    };
    if body.sig.inputs.is_empty()
        || !body.sig.generics.params.is_empty()
        || body.sig.generics.where_clause.is_some()
        || body.sig.unsafety.is_some()
        || body.sig.asyncness.is_some()
        || body.sig.abi.is_some()
    {
        return Err(syn::Error::new_spanned(
            &body.sig,
            "sim_test needs a safe `fn name(sim: SimRoot, ...)`",
        ));
    }
    if let Some(attr) = body
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("should_panic"))
    {
        return Err(syn::Error::new_spanned(
            attr,
            "sim_test does not support should_panic; catch the expected panic inside the test",
        ));
    }
    crate::test::expand_property(body, Some(seeds))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_seed_argument_and_supported_signature() {
        for attr in [quote!(), quote!(seeds = 20), quote!(seeds = 0)] {
            let expanded = expand(
                attr,
                syn::parse_quote!(
                    fn sample(sim: SimRoot) {}
                ),
            )
            .unwrap();
            assert!(expanded.to_string().contains(":: std :: sim :: __run"));
        }
        for attr in [
            quote!(seeds = -1),
            quote!(seeds = "20"),
            quote!(cases = 20),
            quote!(seeds = 20, seeds = 2),
        ] {
            assert!(
                expand(
                    attr,
                    syn::parse_quote!(
                        fn sample(sim: SimRoot) {}
                    )
                )
                .is_err()
            );
        }
        for body in [
            syn::parse_quote!(
                fn sample() {}
            ),
            syn::parse_quote!(
                async fn sample(sim: SimRoot) {}
            ),
            syn::parse_quote!(
                unsafe fn sample(sim: SimRoot) {}
            ),
            syn::parse_quote!(
                fn sample<T>(sim: T) {}
            ),
            syn::parse_quote!(
                #[should_panic]
                fn sample(sim: SimRoot) {}
            ),
        ] {
            assert!(expand(quote!(), body).is_err());
        }
    }
}
