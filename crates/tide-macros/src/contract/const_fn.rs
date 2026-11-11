//! Const-compatible checks use literal panics; sampled expressions stay uncalled.
use super::*;

pub(super) fn expand(mut f: ItemFn, clauses: Vec<Clause>) -> syn::Result<TS> {
    let function = f.sig.ident.to_string();
    let ensures: Vec<_> = clauses.iter().filter(|c| c.kind == "ensures").collect();
    let mut olds = Vec::new();
    let mut expressions = Vec::new();
    for clause in &clauses {
        let value = extract_old(clause.tokens.clone(), clause, &mut olds)?;
        expressions.push((clause, syn::parse2::<Expr>(value)?));
    }
    let mut loops = Loops {
        function: &function,
        olds: &mut olds,
        error: None,
        index: clauses.len(),
        const_: true,
        sampled: false,
    };
    loops.visit_block_mut(&mut f.block);
    let loop_sampled = loops.sampled;
    if let Some(error) = loops.error {
        return Err(error);
    }
    if !olds.is_empty() {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "old snapshots on const fn are not supported; put the contract on a runtime wrapper",
        ));
    }
    let mut requires: Vec<_> = expressions
        .iter()
        .filter(|(c, _)| c.kind == "requires")
        .map(|(c, e)| check(c, quote!(#e), &function, quote!(true)))
        .collect();
    if loop_sampled || clauses.iter().any(|c| c.sampled) {
        requires.insert(0, quote!(#[cfg(all(test, debug_assertions))] ::core::compile_error!("sampled contracts on const fn are not supported in test builds; use a runtime wrapper");));
    }
    if ensures.is_empty() {
        let body = &f.block;
        f.block = if body.stmts.iter().any(|stmt| matches!(stmt, Stmt::Item(_))) {
            Box::new(syn::parse_quote!({ #(#requires)* #body }))
        } else {
            let statements = &body.stmts;
            Box::new(syn::parse_quote!({ #(#requires)* #(#statements)* }))
        };
        return Ok(quote!(#f));
    }
    let mut snapshots = Vec::new();
    let mut replacements = Vec::new();
    for param in &f.sig.inputs {
        let FnArg::Typed(param) = param else {
            return Err(syn::Error::new_spanned(
                param,
                "ensures on a const method is not supported; use a free const function",
            ));
        };
        let Pat::Ident(pat) = param.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &param.pat,
                "const ensures needs identifier parameters; destructure inside the body",
            ));
        };
        let id = &pat.ident;
        if expressions
            .iter()
            .any(|(c, e)| c.kind == "ensures" && references(e, &id.to_string()))
        {
            let snapshot = Ident::new(
                &format!("__tide_const_entry_{}", replacements.len()),
                Span::mixed_site(),
            );
            snapshots.push(quote!(let #snapshot = if ::core::cfg!(debug_assertions) { Some(::std::__contract::const_entry(&#id)) } else { None };));
            replacements.push((id.to_string(), quote!((*#snapshot.as_ref().unwrap()))));
        }
    }
    let mut checks = Vec::new();
    for (clause, mut value) in expressions {
        if clause.kind != "ensures" {
            continue;
        }
        Replace {
            replacements: &replacements,
            changed: false,
        }
        .visit_expr_mut(&mut value);
        checks.push(check(clause, quote!(#value), &function, quote!(true)));
    }
    let result = Ident::new("result", Span::call_site());
    let label = syn::Lifetime::new("'__tide_const_return", Span::mixed_site());
    struct Returns<'a>(&'a syn::Lifetime);
    impl VisitMut for Returns<'_> {
        fn visit_item_mut(&mut self, _: &mut syn::Item) {}
        fn visit_expr_closure_mut(&mut self, _: &mut syn::ExprClosure) {}
        fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
            expr::visit_macro(self, mac);
        }
        fn visit_expr_mut(&mut self, expr: &mut Expr) {
            visit_mut::visit_expr_mut(self, expr);
            if let Expr::Return(value) = expr {
                *expr = Expr::Break(syn::ExprBreak {
                    attrs: value.attrs.clone(),
                    break_token: Default::default(),
                    label: Some(self.0.clone()),
                    expr: value.expr.take(),
                });
            }
        }
    }
    Returns(&label).visit_block_mut(&mut f.block);
    let body = &f.block;
    let annotation = match &f.sig.output {
        ReturnType::Default => quote!(: ()),
        ReturnType::Type(_, ty) if !mentions_impl(ty) => quote!(: #ty),
        _ => quote!(),
    };
    f.block = Box::new(syn::parse_quote!({
        ::std::__contract::const_ensures_marker();
        #(#requires)* #(#snapshots)*
        let #result #annotation = #label: #body;
        #(#checks)* #result
    }));
    Ok(quote!(#f))
}

pub(super) fn check(clause: &Clause, value: TS, function: &str, schedule: TS) -> TS {
    if clause.sampled {
        // Type-check sampled expressions outside const evaluation, without executing them.
        return quote!(let _ = || -> bool { #value };);
    }
    let span = clause.span.unwrap();
    let message = format!(
        "{} violated: {} (in `{function}` at {}:{}:{})",
        clause.kind,
        clause.text,
        span.file(),
        span.line(),
        span.column()
    );
    quote!(if ::core::cfg!(debug_assertions) && (#schedule) && !(#value) { ::core::panic!("{}", #message); })
}
