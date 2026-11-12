use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{FnArg, ItemFn, Pat, Type};

pub fn expand(attr: TokenStream, body: ItemFn) -> syn::Result<TokenStream> {
    if !attr.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            "property tests take no attribute arguments",
        ));
    }
    expand_property(body, None)
}

pub(super) fn expand_property(
    mut body: ItemFn,
    simulation: Option<u64>,
) -> syn::Result<TokenStream> {
    if body.sig.inputs.is_empty() && simulation.is_none() {
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

    let guidance = crate::test_guidance::Guidance::new(&body)?;
    let name = body.sig.ident.clone();
    let source = |start: Span, end: Span| {
        let (file, line, column, end_line, end_column) = if cfg!(test) {
            (String::new(), 0, 0, 0, 0)
        } else {
            let (a, b) = (start.unwrap(), end.unwrap());
            (
                a.file(),
                a.line() as u32,
                a.column() as u32,
                b.line() as u32,
                b.column() as u32,
            )
        };
        quote!(::std::test::Source { file: #file, start: (#line, #column), end: (#end_line, #end_column) })
    };
    let body_source = source(
        body.block.brace_token.span.open(),
        body.block.brace_token.span.close(),
    );
    let call_source = source(name.span(), name.span());
    let excluded = crate::contract::nested_bodies(&body.block)
        .into_iter()
        .map(|(start, end)| {
            if cfg!(test) {
                source(start, end)
            } else {
                source(start.unwrap().start().into(), end.unwrap().end().into())
            }
        })
        .collect::<Vec<_>>();
    let should_panic = body.attrs.iter().any(|a| a.path().is_ident("should_panic"));
    let visibility = body.vis.clone();
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
        if attr.path().is_ident("ignore")
            || attr.path().is_ident("should_panic")
            || attr.path().is_ident("cfg")
            || attr.path().is_ident("cfg_attr")
        {
            harness_attrs.push(attr.clone());
            false
        } else {
            true
        }
    });
    let generator = syn::Ident::new("__tide_gen", Span::mixed_site());
    let mut draws = Vec::new();
    let mut args = Vec::new();
    let mut fixed_seed = false;
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
        let mut draw = Vec::new();
        if simulation.is_some() && index == 0 {
            // Create the simulator only after all ordinary arguments have been drawn.
            args.push(arg);
            draws.push(Vec::new());
            continue;
        }
        if simulation.is_none()
            && label == "seed"
            && matches!(ty.as_ref(), Type::Path(p) if p.path.is_ident("u64"))
        {
            fixed_seed = true;
            draw.push(quote!(let #arg: u64 = #generator.seed();));
        } else if let Some(is_mut) = borrowed {
            let referent = quote!(<#ty as ::core::ops::Deref>::Target);
            let storage = format_ident!("__tide_storage_{index}", span = Span::mixed_site());
            let owned = quote!(<#referent as ::std::test::Borrowed>::Owned);
            let borrow = if is_mut {
                quote!(::core::borrow::BorrowMut::borrow_mut(&mut #storage))
            } else {
                quote!(::core::borrow::Borrow::borrow(&#storage))
            };
            let initial = guidance.draw(index, ty, owned, &generator, &storage, is_mut);
            draw.push(quote! {
                #initial
                let #arg: #ty = #borrow;
            });
        } else {
            draw.push(guidance.draw(index, ty, quote!(#ty), &generator, &arg, false));
        }
        draw.push(quote!(#generator.record(#label, &#arg);));
        draws.push(draw);
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

    let draws = guidance.order.iter().map(|index| &draws[*index]);
    let sim_draw = simulation.map(|_| {
        let sim = &args[0];
        quote!(let #sim = #generator.simulation();)
    });
    let cases = simulation.unwrap_or(100);
    let simulated = simulation.is_some();
    let runner = if simulated {
        quote!(::std::sim::__run)
    } else {
        quote!(::std::test::run_config)
    };
    let call = quote_spanned!(name.span()=> #name(#(#args),*));

    let body = crate::contract::property_body(body)?;
    // The driver reads this nested function's resolved signature during test synthesis.
    Ok(quote! {
        #[::core::prelude::v1::test]
        #(#harness_attrs)*
        #visibility fn #name() {
            #[doc = "__tide_property_body"]
            #body
            #runner(
                ::core::concat!(::core::module_path!(), "::", ::core::stringify!(#name)),
                ::std::test::Config { cases: #cases, body: #body_source, call: #call_source, excluded: &[#(#excluded),*], should_panic: #should_panic, simulation: #simulated, fixed_seed: #fixed_seed },
                |#generator| {
                    #(#(#draws)*)*
                    #sim_draw
                    let #value = #call;
                    #finish
                },
            );
        }
    })
}
