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
    let generated = doc_value(&body, "__tide_generated_contract:");
    if generated.is_some() {
        let mut copied = Vec::new();
        for attr in &body.attrs {
            if let Some(text) = attribute_value(attr, "__tide_requires:") {
                let tokens: TokenStream = text
                    .parse()
                    .map_err(|error| syn::Error::new_spanned(attr, error))?;
                copied.push(syn::parse_quote!(#[::std::prelude::rust_2024::requires(#tokens)]));
            }
            if let Some(text) = attribute_value(attr, "__tide_allow:") {
                use syn::parse::Parser;
                copied.extend(syn::Attribute::parse_outer.parse_str(&text)?);
            }
        }
        body.attrs.extend(copied);
    }
    let allow_vacuous = allow_vacuous(&body)?;
    let plain = body.sig.inputs.is_empty() && simulation.is_none() && generated.is_none();
    // Keep process entry in place for ROOT-5's forbidden harness call.
    // Const ordinary tests stay const-callable and cannot reach replayable effects.
    if plain && (body.sig.constness.is_some()
        || (body.sig.ident == "main"
            && body.attrs.iter().any(|a| a.path().segments.last().is_some_and(|s| s.ident == "main"))))
    {
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

    rewrite_requires(&mut body, generated.as_deref())?;
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
    let capability_simulator = syn::Ident::new("__tide_sim", Span::mixed_site());
    let capabilities = doc_value(&body, "__tide_capabilities:")
        .is_some()
        .then(|| quote!(let #capability_simulator = #generator.simulation();));
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
        let element = doc_value(&body, &format!("__tide_element:{index}:"))
            .map(|text| draw_expression(&text, &generator, &capability_simulator, &source))
            .transpose()?;
        let element = element.map(|e| quote!(#e));
        let custom = doc_value(&body, &format!("__tide_draw:{index}:"));
        if let Some(custom) = custom {
            let expression = draw_expression(&custom, &generator, &capability_simulator, &source)?;
            draw.push(guidance.draw(
                index,
                ty,
                quote!(#ty),
                &generator,
                &arg,
                false,
                Some((quote!(#expression), element)),
            ));
        } else if generated.is_none()
            && simulation.is_none()
            && label == "seed"
            && matches!(ty.as_ref(), Type::Path(p) if p.path.is_ident("u64"))
        {
            fixed_seed = true;
            draw.push(quote!(let #arg: u64 = #generator.seed();));
        } else if let Some(is_mut) = borrowed {
            let referent = quote!(<#ty as ::core::ops::Deref>::Target);
            let storage = format_ident!("__tide_storage_{index}", span = Span::mixed_site());
            let owned = if let Some(concrete) =
                doc_value(&body, &format!("__tide_storage_type:{index}:"))
            {
                let concrete: Type = syn::parse_str(&concrete)?;
                quote!(#concrete)
            } else {
                quote!(<#referent as ::std::test::Borrowed>::Owned)
            };
            let borrow = if is_mut {
                quote!(::core::borrow::BorrowMut::borrow_mut(&mut #storage))
            } else {
                quote!(::core::borrow::Borrow::borrow(&#storage))
            };
            let initial =
                if let Some(custom) = doc_value(&body, &format!("__tide_storage_draw:{index}:")) {
                    let expression =
                        draw_expression(&custom, &generator, &capability_simulator, &source)?;
                    guidance.draw(
                        index,
                        ty,
                        owned,
                        &generator,
                        &storage,
                        is_mut,
                        Some((quote!(#expression), element)),
                    )
                } else {
                    guidance.draw(index, ty, owned, &generator, &storage, is_mut, None)
                };
            draw.push(quote! {
                #initial
                let #arg: #ty = #borrow;
            });
        } else {
            draw.push(guidance.draw(index, ty, quote!(#ty), &generator, &arg, false, None));
        }
        if doc_value(&body, &format!("__tide_opaque:{index}:")).is_none() {
            draw.push(quote!(#generator.record(#label, &#arg);));
        }
        draws.push(draw);
        args.push(arg);
    }

    let value = syn::Ident::new("__tide_result", Span::mixed_site());
    let error = syn::Ident::new("__tide_error", Span::mixed_site());
    // Ordinary tests return the original value; inspect Result layers by borrowing it.
    let mut finish = if plain {
        quote!(::core::result::Result::Ok(()))
    } else {
        quote!(::std::test::property_result(#value))
    };
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
    let cases = simulation.unwrap_or(if plain { 1 } else { 100 });
    let simulated = simulation.is_some();
    let runner = if simulated {
        quote!(::std::sim::__run)
    } else {
        quote!(::std::test::run_config)
    };
    let call = quote_spanned!(name.span()=> #name(#(#args),*));

    let replay_config = [
        format!("__tide_replay_config:cases:{cases}"),
        format!("__tide_replay_config:plain:{plain}"),
        format!("__tide_replay_config:simulation:{simulated}"),
        format!("__tide_replay_config:fixed_seed:{fixed_seed}"),
        format!("__tide_replay_config:should_panic:{should_panic}"),
        format!("__tide_replay_config:allow_vacuous:{allow_vacuous:?}"),
    ];
    let allow_vacuous = match allow_vacuous {
        Some(reason) => quote!(::core::option::Option::Some(#reason)),
        None => quote!(::core::option::Option::None),
    };
    let config = quote!(::std::test::Config { cases: #cases, plain: #plain, body: #body_source, call: #call_source, excluded: &[#(#excluded),*], should_panic: #should_panic, simulation: #simulated, fixed_seed: #fixed_seed, allow_vacuous: #allow_vacuous });
    let signature = if plain { body.sig.clone() } else { syn::parse_quote!(fn #name()) };
    let tracked = body.attrs.iter().filter(|a| plain && a.path().is_ident("track_caller"));
    let tracked = quote!(#(#tracked)*);
    let invocation = if plain {
        quote! {
            ::std::test::run_plain(
                ::core::concat!(::core::module_path!(), "::", ::core::stringify!(#name)),
                #config, #name, |#value| { #finish },
            )
        }
    } else {
        quote! {
            #runner(
                ::core::concat!(::core::module_path!(), "::", ::core::stringify!(#name)),
                #config,
                |#generator| {
                    #capabilities
                    #(#(#draws)*)*
                    #sim_draw
                    let #value = #call;
                    #finish
                },
            )
        }
    };
    let body = crate::contract::property_body(body)?;
    // The driver reads this nested function's resolved signature during test synthesis.
    Ok(quote! {
        #[::core::prelude::v1::test]
        #(#harness_attrs)*
        #(#[doc = #replay_config])*
        #tracked
        #visibility #signature {
            #[doc = "__tide_property_body"]
            #body
            #invocation
        }
    })
}

fn doc_value(body: &ItemFn, prefix: &str) -> Option<String> {
    body.attrs
        .iter()
        .find_map(|attr| attribute_value(attr, prefix))
}
fn attribute_value(attr: &syn::Attribute, prefix: &str) -> Option<String> {
    if attr.path().is_ident("doc")
        && let syn::Meta::NameValue(meta) = &attr.meta
        && let syn::Expr::Lit(value) = &meta.value
        && let syn::Lit::Str(value) = &value.lit
    {
        return value.value().strip_prefix(prefix).map(str::to_owned);
    }
    None
}

fn allow_vacuous(body: &ItemFn) -> syn::Result<Option<String>> {
    use syn::parse::Parser;
    let mut result = None;
    for attr in &body.attrs {
        if !attr.path().is_ident("allow") {
            continue;
        }
        let metas = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
            .parse2(attr.meta.require_list()?.tokens.clone())?;
        if metas.iter().any(|m| m.path().is_ident("vacuous")) {
            let reason = metas.iter().find_map(|m| {
                if let syn::Meta::NameValue(m) = m
                    && m.path.is_ident("reason")
                    && let syn::Expr::Lit(lit) = &m.value
                    && let syn::Lit::Str(reason) = &lit.lit
                {
                    return Some(reason.value());
                }
                None
            });
            result = Some(reason.filter(|s| !s.trim().is_empty()).ok_or_else(|| {
                syn::Error::new_spanned(attr, "allow(vacuous) needs a nonempty reason")
            })?);
        }
    }
    Ok(result)
}

// The clause parser owns instantiation too, including admitted builtin macro arguments.
fn rewrite_requires(body: &mut ItemFn, self_type: Option<&str>) -> syn::Result<()> {
    let Some(self_type) = self_type else {
        return Ok(());
    };
    let mut types = Vec::new();
    if !self_type.is_empty() {
        types.push(("Self".to_owned(), syn::parse_str(self_type)?));
    }
    for attr in &body.attrs {
        if let Some(value) = attribute_value(attr, "__tide_type:")
            && let Some((name, ty)) = value.split_once(':')
        {
            types.push((name.to_owned(), syn::parse_str(ty)?));
        }
    }
    for attr in &mut body.attrs {
        if attr
            .path()
            .segments
            .last()
            .is_some_and(|s| s.ident == "requires")
        {
            let tokens = attr.meta.require_list()?.tokens.clone();
            let instantiated = crate::contract::instantiate_requires(tokens, &types)?;
            if let syn::Meta::List(list) = &mut attr.meta {
                list.tokens = instantiated;
            }
        }
    }
    Ok(())
}

fn draw_expression(
    text: &str,
    generator: &syn::Ident,
    simulator: &syn::Ident,
    source: &impl Fn(Span, Span) -> TokenStream,
) -> syn::Result<syn::Expr> {
    use syn::spanned::Spanned;
    use syn::visit_mut::{self, VisitMut};
    struct Rewrite<'a, F> {
        generator: &'a syn::Ident,
        simulator: &'a syn::Ident,
        source: &'a F,
    }
    impl<F: Fn(Span, Span) -> TokenStream> VisitMut for Rewrite<'_, F> {
        fn visit_ident_mut(&mut self, id: &mut syn::Ident) {
            if id == "__tide_gen" {
                *id = self.generator.clone();
            } else if id == "__tide_sim" {
                *id = self.simulator.clone();
            }
        }
        fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
            visit_mut::visit_expr_call_mut(self, call);
            if matches!(&*call.func, syn::Expr::Path(p) if p.qself.is_none() && p.path.segments.iter().map(|s| s.ident.to_string()).eq(["std", "test", "attempt"].map(str::to_owned)))
            {
                let location = (self.source)(call.span(), call.span());
                call.args.insert(1, syn::parse2(location).unwrap());
            }
        }
    }
    let mut expression = syn::parse_str(text)?;
    Rewrite {
        generator,
        simulator,
        source,
    }
    .visit_expr_mut(&mut expression);
    Ok(expression)
}
