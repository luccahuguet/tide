use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Pat, Type};

pub fn expand(attr: TokenStream, mut body: ItemFn) -> syn::Result<TokenStream> {
    if !attr.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            "property tests take no attribute arguments",
        ));
    }
    if body.sig.inputs.is_empty() {
        return Ok(quote!(#[::core::prelude::v1::test] #body));
    }
    if !body.sig.generics.params.is_empty() || body.sig.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &body.sig.generics,
            "property tests need concrete parameter types",
        ));
    }
    if let Some(token) = body.sig.unsafety {
        return Err(syn::Error::new_spanned(
            token,
            "test functions must be safe",
        ));
    }
    if let Some(token) = body.sig.asyncness {
        return Err(syn::Error::new_spanned(
            token,
            "test functions cannot be async",
        ));
    }

    let name = &body.sig.ident;
    let visibility = &body.vis;
    // The driver's second compilation supplies resolved parameter and result kinds.
    let resolved = body.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("doc")
            && let syn::Meta::NameValue(meta) = &attr.meta
            && let syn::Expr::Lit(value) = &meta.value
            && let syn::Lit::Str(value) = &value.lit
        {
            return value
                .value()
                .strip_prefix("__tide_property_parameters:")
                .map(str::to_owned);
        }
        None
    });
    let (borrows, results) = resolved
        .as_deref()
        .and_then(|kinds| kinds.split_once(':'))
        .map_or((None, 0), |(borrows, results)| {
            (Some(borrows), results.len())
        });
    let mut harness_attrs = Vec::new();
    body.attrs.retain(|attr| {
        if attr.path().is_ident("ignore") || attr.path().is_ident("should_panic") {
            harness_attrs.push(attr.clone());
            false
        } else {
            true
        }
    });
    let generator = syn::Ident::new("__tide_gen", Span::mixed_site());
    let mut draws = Vec::new();
    let mut args = Vec::new();
    for (index, param) in body.sig.inputs.iter().enumerate() {
        let FnArg::Typed(param) = param else {
            return Err(syn::Error::new_spanned(
                param,
                "test functions cannot take self",
            ));
        };
        let arg = format_ident!("__tide_arg_{index}", span = Span::mixed_site());
        let label = match param.pat.as_ref() {
            Pat::Ident(pat) => pat.ident.to_string(),
            _ => format!("arg{}", index + 1),
        };
        let ty = &param.ty;
        let borrowed = match borrows.and_then(|kinds| kinds.as_bytes().get(index)) {
            Some(b'r') => Some(false),
            Some(b'm') => Some(true),
            Some(b'v') => None,
            _ => match ty.as_ref() {
                Type::Reference(reference) => Some(reference.mutability.is_some()),
                _ => None,
            },
        };
        if let Some(is_mut) = borrowed {
            let referent = quote!(<#ty as ::core::ops::Deref>::Target);
            let storage = format_ident!("__tide_storage_{index}", span = Span::mixed_site());
            let owned = quote!(<#referent as ::std::test::Borrowed>::Owned);
            let (mutable, borrow) = if is_mut {
                (
                    quote!(mut),
                    quote!(::core::borrow::BorrowMut::borrow_mut(&mut #storage)),
                )
            } else {
                (quote!(), quote!(::core::borrow::Borrow::borrow(&#storage)))
            };
            draws.push(quote! {
                let #mutable #storage = <#owned as ::std::test::Arbitrary>::arbitrary(#generator);
                let #arg: #ty = #borrow;
            });
        } else {
            draws.push(quote! {
                let #arg = <#ty as ::std::test::Arbitrary>::arbitrary(#generator);
            });
        }
        draws.push(quote!(#generator.record(#label, &#arg);));
        args.push(arg);
    }

    let value = syn::Ident::new("__tide_result", Span::mixed_site());
    let error = syn::Ident::new("__tide_error", Span::mixed_site());
    let mut finish = quote!(::std::test::property_result(#value));
    for _ in 0..results {
        finish = quote! {
            match #value {
                ::core::result::Result::Ok(#value) => #finish,
                ::core::result::Result::Err(#error) => ::core::result::Result::Err(::std::format!("{:?}", #error)),
            }
        };
    }

    // The driver reads this nested function's resolved signature during test synthesis.
    Ok(quote! {
        #[::core::prelude::v1::test]
        #(#harness_attrs)*
        #visibility fn #name() {
            #[doc = "__tide_property_body"]
            #body
            ::std::test::run(
                ::core::concat!(::core::module_path!(), "::", ::core::stringify!(#name)),
                |#generator| {
                    #(#draws)*
                    let #value = #name(#(#args),*);
                    #finish
                },
            );
        }
    })
}
