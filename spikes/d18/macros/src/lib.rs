//! D18 spike: Tide's contract and entry macros as Rust proc macros (throwaway).
//!
//! - `#[requires(e)]`, `#[ensures(e)]`: the first contract attribute on a fn
//!   takes every contract attribute of that fn, so `requires` checks run
//!   before `old` snapshots whatever the attribute order. Checks are
//!   `if cfg!(debug_assertions)`, so release builds type-check clauses but never
//!   evaluate them; clause tokens may use `==>`, `result`,
//!   `old(e)`, and bounded `forall`/`exists`.
//! - `#[invariants]` (or any contract attribute) rewrites every `for`/`while`
//!   loop whose body starts with `invariant!(e);` so the invariant is checked
//!   at entry, after each iteration, and at exit, with the next-index rule.
//! - `#[tide::main]` lets `fn main(root: Root)` be written in a bin crate.

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream as TS, TokenTree};
use quote::{ToTokens, quote};
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ItemFn, Pat, RangeLimits, ReturnType, Stmt, Type, parse_macro_input};

#[proc_macro_attribute]
pub fn requires(attr: TokenStream, item: TokenStream) -> TokenStream {
    contract(Some(("requires", attr.into())), item)
}

#[proc_macro_attribute]
pub fn ensures(attr: TokenStream, item: TokenStream) -> TokenStream {
    contract(Some(("ensures", attr.into())), item)
}

#[proc_macro_attribute]
pub fn invariants(_attr: TokenStream, item: TokenStream) -> TokenStream {
    contract(None, item)
}

/// Reached only when no enclosing contract attribute rewrote the loop.
#[proc_macro]
pub fn invariant(_: TokenStream) -> TokenStream {
    quote!(::core::compile_error!(
        "invariant!(..) must be the first statement of a loop body in a fn marked #[invariants], #[requires], or #[ensures]"
    ))
    .into()
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let f = parse_macro_input!(item as ItemFn);
    let (attrs, inputs, output, block) = (&f.attrs, &f.sig.inputs, &f.sig.output, &f.block);
    let arg = if inputs.is_empty() { quote!() } else { quote!(::std::__rt::root()) };
    quote! {
        #(#attrs)*
        fn main() -> ::std::process::ExitCode {
            fn __tide_main(#inputs) #output #block
            ::std::process::Termination::report(__tide_main(#arg))
        }
    }
    .into()
}

fn contract(first: Option<(&str, TS)>, item: TokenStream) -> TokenStream {
    let mut f: ItemFn = match syn::parse(item) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
    };
    let (mut reqs, mut enss) = (Vec::new(), Vec::new());
    let mut take = |kind: &str, ts: TS| match kind {
        "requires" => reqs.push(ts),
        _ => enss.push(ts),
    };
    if let Some((k, ts)) = first {
        take(k, ts);
    }
    let mut keep = Vec::new();
    for a in std::mem::take(&mut f.attrs) {
        let name = a.path().segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
        match name.as_str() {
            "requires" | "ensures" => match a.meta.require_list() {
                Ok(l) => take(&name, l.tokens.clone()),
                Err(e) => return e.to_compile_error().into(),
            },
            "invariants" => {}
            _ => keep.push(a),
        }
    }
    f.attrs = keep;
    let fname = f.sig.ident.to_string();
    Loops { fname: fname.clone() }.visit_block_mut(&mut f.block);

    let req_checks = reqs.iter().map(|c| check("requires", c, &fname));
    let mut olds = Vec::new();
    let ens: Vec<TS> = enss.iter().map(|c| extract_old(c.clone(), &mut olds)).collect();
    let ens_checks = ens.iter().zip(&enss).map(|(lowered, text)| check_lowered("ensures", lowered, text, &fname));
    let snapshots = olds.iter().map(|(id, e)| {
        quote!(let #id = if ::core::cfg!(debug_assertions) { ::core::option::Option::Some((#e).clone()) } else { ::core::option::Option::None };)
    });
    let body = &f.block;
    let new_body = if enss.is_empty() {
        quote!({ #(#req_checks)* #body })
    } else {
        let result = Ident::new("result", Span::call_site());
        let ann = match &f.sig.output {
            ReturnType::Type(_, t) if !mentions_impl(t) => quote!(-> #t),
            ReturnType::Type(..) => quote!(),
            ReturnType::Default => quote!(-> ()),
        };
        quote!({
            #(#req_checks)*
            #(#snapshots)*
            #[allow(clippy::redundant_closure_call)]
            let #result = (|| #ann #body)();
            #(#ens_checks)*
            #result
        })
    };
    f.block = Box::new(syn::parse2(new_body).expect("contract body"));
    f.into_token_stream().into()
}

fn mentions_impl(t: &Type) -> bool {
    t.to_token_stream().into_iter().any(|t| matches!(t, TokenTree::Ident(i) if i == "impl"))
}

fn check(kind: &str, clause: &TS, fname: &str) -> TS {
    check_lowered(kind, clause, clause, fname)
}

/// `lowered` is the clause after `old` substitution; `text` names it in the message.
fn check_lowered(kind: &str, lowered: &TS, text: &TS, fname: &str) -> TS {
    let e = lower(lowered.clone());
    let msg = format!("{kind} violated: {text} (in `{fname}`)");
    quote!(if ::core::cfg!(debug_assertions) && !(#e) { ::core::panic!("{}", #msg); })
}

/// Replace each `old(e)` with a fresh local snapshotted at entry.
fn extract_old(ts: TS, olds: &mut Vec<(Ident, TS)>) -> TS {
    let toks: Vec<TokenTree> = ts.into_iter().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        let after_dot = matches!(out.last(), Some(TokenTree::Punct(p)) if p.as_char() == '.');
        match (&toks[i], toks.get(i + 1)) {
            (TokenTree::Ident(id), Some(TokenTree::Group(g)))
                if id == "old" && g.delimiter() == Delimiter::Parenthesis && !after_dot =>
            {
                let name = Ident::new(&format!("__tide_old_{}", olds.len()), Span::mixed_site());
                olds.push((name.clone(), g.stream()));
                out.extend(quote!((*#name.as_ref().unwrap())));
                i += 2;
                continue;
            }
            (TokenTree::Group(g), _) => out.push(regroup(g, extract_old(g.stream(), olds))),
            (t, _) => out.push(t.clone()),
        }
        i += 1;
    }
    out.into_iter().collect()
}

fn regroup(g: &Group, ts: TS) -> TokenTree {
    let mut n = Group::new(g.delimiter(), ts);
    n.set_span(g.span());
    TokenTree::Group(n)
}

/// Lower clause syntax (`==>`, `forall`, `exists`) to a Rust bool expression.
fn lower(ts: TS) -> TS {
    let toks: Vec<TokenTree> = ts
        .into_iter()
        .map(|t| match t {
            TokenTree::Group(g) => regroup(&g, lower(g.stream())),
            t => t,
        })
        .collect();
    // A quantifier body extends to the end of the sequence.
    for k in 0..toks.len() {
        let quant = matches!(&toks[k], TokenTree::Ident(i) if i == "forall" || i == "exists");
        if quant && is_punct(toks.get(k + 1), '|') {
            let Some(close) = (k + 2..toks.len()).find(|&j| is_punct(toks.get(j), '|')) else { break };
            let q = quantifier(&toks[k].to_string(), &toks[k + 2..close], &toks[close + 1..]);
            let mut seq = toks[..k].to_vec();
            seq.push(TokenTree::Group(Group::new(Delimiter::Parenthesis, q)));
            return implication(&seq);
        }
    }
    implication(&toks)
}

fn is_punct(t: Option<&TokenTree>, c: char) -> bool {
    matches!(t, Some(TokenTree::Punct(p)) if p.as_char() == c)
}

fn seq(toks: &[TokenTree]) -> TS {
    toks.iter().cloned().collect()
}

/// `a ==> b` is `!a || b`, lowest precedence, right-associative.
fn implication(toks: &[TokenTree]) -> TS {
    let at = (0..toks.len().saturating_sub(2))
        .find(|&i| is_punct(toks.get(i), '=') && is_punct(toks.get(i + 1), '=') && is_punct(toks.get(i + 2), '>'));
    match at {
        Some(i) => {
            let (lhs, rhs) = (seq(&toks[..i]), implication(&toks[i + 3..]));
            quote!((!(#lhs) || (#rhs)))
        }
        None => seq(toks),
    }
}

/// Split at top-level `&&`.
fn conjuncts(toks: &[TokenTree]) -> Vec<Vec<TokenTree>> {
    let mut parts = vec![Vec::new()];
    let mut i = 0;
    while i < toks.len() {
        if is_punct(toks.get(i), '&') && is_punct(toks.get(i + 1), '&') {
            parts.push(Vec::new());
            i += 2;
            continue;
        }
        parts.last_mut().unwrap().push(toks[i].clone());
        i += 1;
    }
    parts
}

fn is_ident(t: &[TokenTree], name: &Ident) -> bool {
    matches!(t, [TokenTree::Ident(i)] if i == name)
}

/// `forall |i: T| guard ==> p`, `exists |i: T| guard && p`, where guard is
/// `lo <= i && i < hi` (or `i < hi`, `i <= hi`, `c.contains(&i)`).
fn quantifier(kind: &str, binder: &[TokenTree], body: &[TokenTree]) -> TS {
    let Some(TokenTree::Ident(var)) = binder.first() else {
        return quote!(::core::compile_error!("quantifier needs a named binder"));
    };
    let ty = if binder.len() > 2 { seq(&binder[2..]) } else { quote!(_) };
    // forall: guard conjuncts, then the body after `==>`; exists: all conjuncts.
    let (parts, forall_body) = if kind == "forall" {
        let at = (0..body.len().saturating_sub(2))
            .find(|&i| is_punct(body.get(i), '=') && is_punct(body.get(i + 1), '=') && is_punct(body.get(i + 2), '>'));
        let Some(at) = at else {
            return quote!(::core::compile_error!("unbounded forall: write `forall |i: T| lo <= i && i < hi ==> p`"));
        };
        (conjuncts(&body[..at]), Some(implication(&body[at + 3..])))
    } else {
        (conjuncts(body), None)
    };
    let mut rest = parts.as_slice();
    let mut lo = quote!(0);
    // lo <= i
    if let Some(first) = rest.first() {
        if let Some(p) = (0..first.len()).find(|&j| is_punct(first.get(j), '<') && is_punct(first.get(j + 1), '=')) {
            if is_ident(&first[p + 2..], var) {
                lo = seq(&first[..p]);
                rest = &rest[1..];
            }
        }
    }
    // i < hi | i <= hi | c.contains(&i)
    let Some(bound) = rest.first() else {
        return quote!(::core::compile_error!("unbounded quantifier"));
    };
    let iter = if bound.len() > 2 && matches!(&bound[0], TokenTree::Ident(i) if i == var) && is_punct(bound.get(1), '<') {
        if is_punct(bound.get(2), '=') {
            let hi = seq(&bound[3..]);
            quote!((#lo..=#hi))
        } else {
            let hi = seq(&bound[2..]);
            quote!((#lo..#hi))
        }
    } else if bound.len() >= 3
        && matches!(&bound[bound.len() - 2], TokenTree::Ident(i) if i == "contains")
        && is_punct(bound.get(bound.len() - 3), '.')
    {
        let c = seq(&bound[..bound.len() - 3]);
        let x = Ident::new("__tide_x", Span::mixed_site());
        let pred = predicate(&rest[1..], &forall_body);
        let call = if kind == "forall" { quote!(all) } else { quote!(any) };
        return quote!((&#c).into_iter().#call(|#x| { let #var: #ty = ::core::clone::Clone::clone(#x); #pred }));
    } else {
        return quote!(::core::compile_error!("unbounded quantifier: guard must be `lo <= i && i < hi` or `c.contains(&i)`"));
    };
    let pred = predicate(&rest[1..], &forall_body);
    let call = if kind == "forall" { quote!(all) } else { quote!(any) };
    quote!(#iter.#call(|#var: #ty| #pred))
}

/// The quantified predicate: remaining guards imply the forall body, or the
/// remaining conjuncts of an exists.
fn predicate(parts: &[Vec<TokenTree>], forall_body: &Option<TS>) -> TS {
    let ps = parts.iter().map(|p| implication(p));
    let conj = quote!((true #(&& (#ps))*));
    match forall_body {
        Some(b) => quote!((!#conj || (#b))),
        None => conj,
    }
}

/// Rewrites loops whose body starts with `invariant!(e);`.
struct Loops {
    fname: String,
}

impl Loops {
    fn take_invariants(&self, body: &mut syn::Block) -> Option<TS> {
        let mut checks = Vec::new();
        while let Some(Stmt::Macro(m)) = body.stmts.first() {
            if !m.mac.path.is_ident("invariant") {
                break;
            }
            checks.push(check("invariant", &m.mac.tokens, &self.fname));
            body.stmts.remove(0);
        }
        (!checks.is_empty()).then(|| quote!(#(#checks)*))
    }
}

impl VisitMut for Loops {
    fn visit_item_mut(&mut self, _: &mut syn::Item) {}

    fn visit_expr_mut(&mut self, e: &mut Expr) {
        visit_mut::visit_expr_mut(self, e);
        let new = match e {
            Expr::ForLoop(fl) => {
                let Some(check) = self.take_invariants(&mut fl.body) else { return };
                let (label, pat, stmts) = (&fl.label, &fl.pat, &fl.body.stmts);
                match &*fl.expr {
                    Expr::Range(r) if matches!(r.limits, RangeLimits::HalfOpen(_)) && r.start.is_some() && r.end.is_some() => {
                        let (a, b) = (&r.start, &r.end);
                        let (i, end) = (Ident::new("__tide_i", Span::mixed_site()), Ident::new("__tide_end", Span::mixed_site()));
                        // The invariant sees the loop variable as the next index.
                        let bind = match &**pat {
                            Pat::Ident(p) => {
                                let id = &p.ident;
                                quote!(#[allow(unused_variables)] let #id = #i;)
                            }
                            _ => quote!(),
                        };
                        quote!({
                            let mut #i = #a;
                            let #end = #b;
                            #label loop {
                                { #bind #check }
                                if !(#i < #end) { break; }
                                #[allow(unused_variables)]
                                let #pat = #i;
                                #i += 1;
                                #(#stmts)*
                            }
                        })
                    }
                    iter => {
                        let it = Ident::new("__tide_it", Span::mixed_site());
                        quote!({
                            let mut #it = ::core::iter::IntoIterator::into_iter(#iter);
                            #label loop {
                                #check
                                let ::core::option::Option::Some(#pat) = ::core::iter::Iterator::next(&mut #it) else { break; };
                                #(#stmts)*
                            }
                        })
                    }
                }
            }
            Expr::While(wl) => {
                let Some(check) = self.take_invariants(&mut wl.body) else { return };
                let (label, stmts) = (&wl.label, &wl.body.stmts);
                let test = match &*wl.cond {
                    Expr::Let(l) => {
                        let (p, x) = (&l.pat, &l.expr);
                        quote!(let #p = #x else { break; };)
                    }
                    c => quote!(if !(#c) { break; }),
                };
                quote!(#label loop { #check #test #(#stmts)* })
            }
            _ => return,
        };
        *e = syn::parse2(new).expect("loop rewrite");
    }
}
