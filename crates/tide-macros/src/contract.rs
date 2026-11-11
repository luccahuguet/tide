//! One parser/lowering owner for executable clauses and future clause-guided tests.
mod const_fn;
mod expr;
use proc_macro2::{Ident, Span, TokenStream as TS, TokenTree};
use quote::{ToTokens, quote};
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, FnArg, ItemFn, Pat, RangeLimits, ReturnType, Stmt, Type};

#[derive(Clone)]
struct Clause {
    tokens: TS,
    text: String,
    span: Span,
    kind: &'static str,
    index: usize,
    conjunct: usize,
    sampled: bool,
}
impl Clause {
    fn new(kind: &'static str, tokens: TS, span: Span, index: usize) -> syn::Result<Self> {
        let expr = match expr::parse(tokens.clone()) {
            Ok(expr) => expr,
            Err(_)
                if tokens
                    .clone()
                    .into_iter()
                    .any(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == ',')) =>
            {
                return Err(syn::Error::new(
                    span,
                    "FX0804: one expression per clause; split into repeated clauses",
                ));
            }
            Err(error) => return Err(error),
        };
        if kind != "ensures" && references(&expr, "result") {
            return Err(syn::Error::new(
                span,
                "FX0805: `result` is only valid in ensures",
            ));
        }
        let mut class = Class(false);
        class.visit_expr_mut(&mut expr.clone());
        Ok(Self {
            text: source(&tokens, span),
            tokens,
            span,
            kind,
            index,
            conjunct: 0,
            sampled: class.0,
        })
    }
    fn metadata(&self, function: &str) -> TS {
        let Self {
            text,
            span,
            kind,
            index,
            conjunct,
            ..
        } = self;
        let kind = Ident::new(
            match *kind {
                "requires" => "Requires",
                "ensures" => "Ensures",
                _ => "Invariant",
            },
            Span::mixed_site(),
        );
        let native = span.unwrap();
        let (file, line, column) = (native.file(), native.line() as u32, native.column() as u32);
        quote!(::std::__contract::Clause { kind: ::std::__contract::Kind::#kind, text: #text,
            function: #function, file: #file, line: #line, column: #column, index: #index, conjunct: #conjunct })
    }
    fn enabled(&self) -> TS {
        if self.sampled {
            quote!(::core::cfg!(debug_assertions) && ::core::cfg!(test))
        } else {
            quote!(::core::cfg!(debug_assertions))
        }
    }
    fn check(&self, expr: TS, function: &str, schedule: TS) -> TS {
        let metadata = self.metadata(function);
        let helper = Ident::new(self.kind, Span::mixed_site());
        let enabled = self.enabled();
        let skip = if self.sampled {
            quote!(if ::core::cfg!(debug_assertions) && !::core::cfg!(test) {
                ::std::__contract::unchecked(#metadata, "sampled clause outside test crate");
            })
        } else {
            quote!()
        };
        quote! { if (#enabled) && (#schedule) { ::std::__contract::#helper(#metadata, || #expr); } #skip }
    }
}

fn source(tokens: &TS, span: Span) -> String {
    // Attribute call-site spans include the delimiters; sibling clauses supply their group span.
    if let Some(source) = span.source_text() {
        if let (Some(start), Some(end)) = (source.find('('), source.rfind(')')) {
            return source[start + 1..end].to_owned();
        }
    }
    let mut it = tokens.clone().into_iter();
    if let Some(first) = it.next() {
        let last = it.last().unwrap_or_else(|| first.clone());
        if let Some(span) = first.span().join(last.span()) {
            if let Some(text) = span.source_text() {
                return text;
            }
        }
    }
    tokens.to_string()
}

fn conjunct_source(tokens: &TS, clause: &Clause) -> String {
    let native = clause.span.unwrap();
    let Some(source) = native.source_text() else {
        return tokens.to_string();
    };
    let mut tokens = tokens.clone().into_iter();
    let Some(first) = tokens.next() else {
        return String::new();
    };
    let last = tokens.last().unwrap_or_else(|| first.clone());
    let start = first.span().unwrap().start();
    let end = last.span().unwrap().end();
    let offset = |target: proc_macro::Span| {
        let (mut line, mut column) = (native.line(), native.column());
        for (offset, c) in source.char_indices() {
            if (line, column) == (target.line(), target.column()) {
                return Some(offset);
            }
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        ((line, column) == (target.line(), target.column())).then_some(source.len())
    };
    offset(start)
        .zip(offset(end))
        .and_then(|(a, b)| source.get(a..b))
        .unwrap_or(&clause.text)
        .to_owned()
}

pub fn expand(kind: &'static str, tokens: TS, item: TS) -> syn::Result<TS> {
    let mut f = match syn::parse2::<ItemFn>(item.clone()) {
        Ok(f) => f,
        Err(e) => {
            if syn::parse2::<syn::TraitItemFn>(item.clone()).is_ok() {
                return Ok(item);
            }
            return Err(e);
        }
    };
    // Test macros must see the unexpanded clauses, regardless of attribute order.
    if let Some(i) = f
        .attrs
        .iter()
        .position(|a| matches!(attr_name(a).as_str(), "test" | "sim_test"))
    {
        let test = f.attrs.remove(i);
        let name = Ident::new(kind, Span::call_site());
        return Ok(quote!(#test #[::std::prelude::rust_2024::#name(#tokens)] #f));
    }
    if kind == "invariants" && !tokens.is_empty() {
        return Err(syn::Error::new_spanned(
            tokens,
            "invariants takes no arguments",
        ));
    }
    let mut clauses = Vec::new();
    if kind != "invariants" {
        clauses.push(Clause::new(kind, tokens, Span::call_site(), 0)?);
    }
    let mut keep = Vec::new();
    for a in std::mem::take(&mut f.attrs) {
        match attr_name(&a).as_str() {
            "requires" | "ensures" => {
                let k = if attr_name(&a) == "requires" {
                    "requires"
                } else {
                    "ensures"
                };
                let list = a.meta.require_list()?;
                clauses.push(Clause::new(
                    k,
                    list.tokens.clone(),
                    list.delimiter.span().join(),
                    clauses.len(),
                )?);
            }
            "invariants" => {
                if let syn::Meta::List(list) = &a.meta {
                    if !list.tokens.is_empty() {
                        return Err(syn::Error::new_spanned(a, "invariants takes no arguments"));
                    }
                }
            }
            _ => keep.push(a),
        }
    }
    f.attrs = keep;
    let function = f.sig.ident.to_string();
    let ensures: Vec<_> = clauses
        .iter()
        .filter(|c| c.kind == "ensures")
        .cloned()
        .collect();
    if !ensures.is_empty()
        && f.sig.inputs.iter().any(|p| match p {
            FnArg::Typed(p) => bindings(&p.pat).iter().any(|id| id == "result"),
            _ => false,
        })
    {
        return Err(syn::Error::new_spanned(
            &f.sig.inputs,
            "FX0805: a function with ensures cannot have a parameter named result",
        ));
    }
    let mut requires = Vec::new();
    if f.sig.constness.is_some() {
        return const_fn::expand(f, clauses);
    }
    for clause in clauses.iter().filter(|c| c.kind == "requires") {
        let expr = syn::parse2::<Expr>(lower(clause.tokens.clone())?)?;
        let mut parts = Vec::new();
        requires_conjuncts(&expr, &mut parts);
        for (i, part) in parts.iter().enumerate() {
            let mut c = clause.clone();
            c.conjunct = i;
            let tokens = part.to_token_stream();
            if parts.len() > 1 {
                c.text = conjunct_source(&tokens, clause);
            }
            requires.push(c.check(tokens, &function, quote!(true)));
        }
    }
    let user_track_caller = f.attrs.iter().any(|a| a.path().is_ident("track_caller"));
    if clauses.iter().any(|c| c.kind == "requires") && !user_track_caller {
        f.attrs.push(syn::parse_quote!(#[track_caller]));
    }
    let mut olds = Vec::new();
    let mut post = Vec::new();
    for clause in &ensures {
        post.push((
            clause,
            extract_old(clause.tokens.clone(), clause, &mut olds)?,
        ));
    }
    let mut loops = Loops {
        function: &function,
        olds: &mut olds,
        error: None,
        index: clauses.len(),
        const_: false,
        sampled: false,
    };
    loops.visit_block_mut(&mut f.block);
    if let Some(e) = loops.error {
        return Err(e);
    }
    let mut entry_snapshots = Vec::new();
    let mut replacements = Vec::new();
    for param in &f.sig.inputs {
        let (ids, mutable) = match param {
            FnArg::Typed(p) => (
                bindings(&p.pat),
                matches!(expr::peel_type(&p.ty), Type::Reference(r) if r.mutability.is_some()),
            ),
            FnArg::Receiver(r) => (
                vec![r.self_token.into()],
                matches!(expr::peel_type(&r.ty), Type::Reference(r) if r.mutability.is_some()),
            ),
        };
        if mutable {
            continue;
        }
        for id in &ids {
            let uses: Vec<_> = post
                .iter()
                .filter(|(_, t)| {
                    references(
                        &syn::parse2(lower(t.clone()).unwrap()).unwrap(),
                        &id.to_string(),
                    )
                })
                .map(|(c, _)| *c)
                .collect();
            if uses.is_empty() {
                continue;
            }
            let snapshot = Ident::new(
                &format!("__tide_entry_{}", replacements.len()),
                Span::mixed_site(),
            );
            let enabled: Vec<_> = uses.iter().map(|c| c.enabled()).collect();
            let meta = uses[0].metadata(&function);
            entry_snapshots.push(quote! {
            let #snapshot = if false #(|| (#enabled))* {
                    use ::std::__contract::{CopyEntry as _, CloneEntry as _, BorrowEntry as _, MutableAliasEntry as _};
                    ::core::option::Option::Some(::std::__contract::entry(#meta, || {
                        let snapshot = (&&&::std::__contract::Entry(&#id)).snapshot();
                        ::std::__contract::supported_entry(&snapshot);
                        snapshot
                    }))
            } else { ::core::option::Option::None };
        });
            replacements.push((
                id.to_string(),
                quote!((*::core::ops::Deref::deref(#snapshot.as_ref().unwrap()))),
            ));
        }
    }
    let mut checks = Vec::new();
    for (clause, tokens) in post {
        let mut expr: Expr = syn::parse2(lower(tokens)?)?;
        Replace {
            replacements: &replacements,
            changed: false,
        }
        .visit_expr_mut(&mut expr);
        checks.push(clause.check(quote!(#expr), &function, quote!(true)));
    }
    let snapshots = olds.iter().map(|(id, expr, clause)| {
        let enabled = clause.enabled();
        let meta = clause.metadata(&function);
        quote!(let #id = if #enabled { ::core::option::Option::Some(::std::__contract::old(#meta, || {
            use ::std::__contract::{RefOld as _, ValueOld as _};
            (&&::std::__contract::Old(&(#expr))).snapshot()
        })) } else { ::core::option::Option::None };)
    });
    let body = &f.block;
    let body = if ensures.is_empty() && (requires.is_empty() || user_track_caller) {
        if body.stmts.iter().any(|stmt| matches!(stmt, Stmt::Item(_))) {
            quote!({ #(#requires)* #(#snapshots)* #body })
        } else {
            let statements = &body.stmts;
            quote!({ #(#requires)* #(#snapshots)* #(#statements)* })
        }
    } else {
        let result = Ident::new("result", Span::call_site());
        let ann = match &f.sig.output {
            ReturnType::Type(_, t) if !mentions_impl(t) => quote!(-> #t),
            ReturnType::Type(..) => quote!(),
            ReturnType::Default => quote!(-> ()),
        };
        let mut args = Vec::new();
        let mut params = Vec::new();
        let mut types = Vec::new();
        let mut body = body.clone();
        for param in &f.sig.inputs {
            match param {
                FnArg::Typed(p) => {
                    if let Type::Reference(r) = expr::peel_type(&p.ty) {
                        if r.mutability.is_some() {
                            let mut ty = r.clone();
                            ty.lifetime = None;
                            let pat = &p.pat;
                            if let Pat::Ident(p) = pat.as_ref() {
                                let id = &p.ident;
                                args.push(quote!(&mut *#id));
                                params.push(quote!(#pat));
                                types.push(quote!(#ty));
                            }
                        }
                    }
                }
                FnArg::Receiver(r) if matches!(expr::peel_type(&r.ty), Type::Reference(t) if t.mutability.is_some()) =>
                {
                    let id = Ident::new("__tide_self", Span::mixed_site());
                    Receiver(&id).visit_block_mut(&mut body);
                    let mutability = r.mutability;
                    let Type::Reference(mut ty) = expr::peel_type(&r.ty).clone() else {
                        unreachable!()
                    };
                    ty.lifetime = None;
                    args.push(quote!(&mut *self));
                    params.push(quote!(#mutability #id));
                    types.push(quote!(#ty));
                }
                _ => (),
            }
        }
        let input_type = if types
            .iter()
            .any(|t| syn::parse2::<Type>(t.clone()).is_ok_and(|t| mentions_impl(&t)))
        {
            quote!()
        } else {
            quote!(: (#(#types,)*))
        };
        let call =
            quote!(::std::__contract::call(|(#(#params,)*) #input_type| #ann #body, (#(#args,)*)));
        let returned = if ensures.is_empty() {
            call
        } else if matches!(&f.sig.output, ReturnType::Type(_, ty) if matches!(ty.as_ref(), Type::Never(_)))
        {
            quote! {
                let #result = #call;
                #[allow(unreachable_code)]
                { #(#checks)* #result }
            }
        } else {
            quote!(let #result = #call; #(#checks)* #result)
        };
        quote!({
            #(#requires)* #(#snapshots)* #(#entry_snapshots)*
            #returned
        })
    };
    f.block = Box::new(syn::parse2(body)?);
    Ok(quote!(#f))
}

fn attr_name(a: &syn::Attribute) -> String {
    a.path()
        .segments
        .last()
        .map_or(String::new(), |s| s.ident.to_string())
}
fn requires_conjuncts<'a>(expr: &'a Expr, parts: &mut Vec<&'a Expr>) {
    if let Expr::Binary(b) = expr {
        if matches!(b.op, syn::BinOp::And(_)) {
            requires_conjuncts(&b.left, parts);
            requires_conjuncts(&b.right, parts);
            return;
        }
    }
    parts.push(expr);
}
fn bindings(pat: &Pat) -> Vec<Ident> {
    struct Bindings(Vec<Ident>);
    impl<'ast> syn::visit::Visit<'ast> for Bindings {
        fn visit_pat_ident(&mut self, pat: &'ast syn::PatIdent) {
            self.0.push(pat.ident.clone());
            syn::visit::visit_pat_ident(self, pat);
        }
    }
    let mut found = Bindings(Vec::new());
    syn::visit::Visit::visit_pat(&mut found, pat);
    found.0
}
fn mentions_impl(ty: &Type) -> bool {
    struct Impl(bool);
    impl<'ast> syn::visit::Visit<'ast> for Impl {
        fn visit_type_impl_trait(&mut self, _: &'ast syn::TypeImplTrait) {
            self.0 = true;
        }
    }
    let mut found = Impl(false);
    syn::visit::Visit::visit_type(&mut found, ty);
    found.0
}
struct Receiver<'a>(&'a Ident);
impl VisitMut for Receiver<'_> {
    fn visit_item_mut(&mut self, _: &mut syn::Item) {}
    fn visit_expr_path_mut(&mut self, path: &mut syn::ExprPath) {
        if path.path.is_ident("self") {
            let id = self.0;
            *path = syn::parse_quote!(#id);
        } else {
            visit_mut::visit_expr_path_mut(self, path);
        }
    }
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        let Ok(mut args) = expr::Arguments::parse(mac) else {
            return;
        };
        args.visit(self);
        let receiver = self.0;
        args.expose_captures(&[("self".into(), quote!(&*#receiver))]);
        mac.tokens = args.tokens();
    }
}
fn references(expr: &Expr, name: &str) -> bool {
    let mut replace = Replace {
        replacements: &[(name.to_owned(), quote!(()))],
        changed: false,
    };
    replace.visit_expr_mut(&mut expr.clone());
    replace.changed
}
struct Replace<'a> {
    replacements: &'a [(String, TS)],
    changed: bool,
}
impl Replace<'_> {
    fn condition(&mut self, expr: &mut Expr) -> Vec<Ident> {
        match expr {
            Expr::Let(value) => {
                self.visit_expr_mut(&mut value.expr);
                let pat = &value.pat;
                bindings(pat)
            }
            Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
                let left = self.condition(&mut binary.left);
                let replacements: Vec<_> = self
                    .replacements
                    .iter()
                    .filter(|(name, _)| !left.iter().any(|id| id == name))
                    .cloned()
                    .collect();
                let mut inner = Replace {
                    replacements: &replacements,
                    changed: false,
                };
                let right = inner.condition(&mut binary.right);
                self.changed |= inner.changed;
                [left, right].concat()
            }
            _ => {
                self.visit_expr_mut(expr);
                Vec::new()
            }
        }
    }
    fn scoped_block(&mut self, bound: Vec<Ident>, block: &mut syn::Block) {
        let replacements: Vec<_> = self
            .replacements
            .iter()
            .filter(|(name, _)| !bound.iter().any(|id| id == name))
            .cloned()
            .collect();
        let mut inner = Replace {
            replacements: &replacements,
            changed: false,
        };
        inner.visit_block_mut(block);
        self.changed |= inner.changed;
    }
    fn scoped(&mut self, bound: Vec<Ident>, expr: &mut Expr) {
        let replacements: Vec<_> = self
            .replacements
            .iter()
            .filter(|(name, _)| !bound.iter().any(|id| id == name))
            .cloned()
            .collect();
        let mut inner = Replace {
            replacements: &replacements,
            changed: false,
        };
        inner.visit_expr_mut(expr);
        self.changed |= inner.changed;
    }
}
impl VisitMut for Replace<'_> {
    fn visit_field_value_mut(&mut self, field: &mut syn::FieldValue) {
        let mut inner = Replace {
            replacements: self.replacements,
            changed: false,
        };
        inner.visit_expr_mut(&mut field.expr);
        if inner.changed && field.colon_token.is_none() {
            field.colon_token = Some(Default::default());
        }
        self.changed |= inner.changed;
    }
    fn visit_expr_if_mut(&mut self, expr: &mut syn::ExprIf) {
        let bound = self.condition(&mut expr.cond);
        self.scoped_block(bound, &mut expr.then_branch);
        if let Some((_, body)) = &mut expr.else_branch {
            self.visit_expr_mut(body);
        }
    }
    fn visit_expr_while_mut(&mut self, expr: &mut syn::ExprWhile) {
        let bound = self.condition(&mut expr.cond);
        self.scoped_block(bound, &mut expr.body);
    }
    fn visit_expr_for_loop_mut(&mut self, expr: &mut syn::ExprForLoop) {
        self.visit_expr_mut(&mut expr.expr);
        self.scoped_block(bindings(&expr.pat), &mut expr.body);
    }
    fn visit_expr_mut(&mut self, e: &mut Expr) {
        if let Expr::Path(p) = e {
            if let Some(id) = p.path.get_ident() {
                if let Some((_, value)) = self.replacements.iter().find(|(name, _)| id == name) {
                    *e = syn::parse2(value.clone()).unwrap();
                    self.changed = true;
                    return;
                }
            }
        }
        visit_mut::visit_expr_mut(self, e);
    }
    fn visit_expr_closure_mut(&mut self, e: &mut syn::ExprClosure) {
        let bound = e.inputs.iter().flat_map(bindings).collect();
        self.scoped(bound, &mut e.body);
    }
    fn visit_arm_mut(&mut self, arm: &mut syn::Arm) {
        let pat = &arm.pat;
        if let Some((_, guard)) = &mut arm.guard {
            self.scoped(bindings(pat), guard);
        }
        self.scoped(bindings(pat), &mut arm.body);
    }
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let mut replacements = self.replacements.to_vec();
        for stmt in &mut block.stmts {
            let mut inner = Replace {
                replacements: &replacements,
                changed: false,
            };
            inner.visit_stmt_mut(stmt);
            self.changed |= inner.changed;
            if let Stmt::Local(local) = stmt {
                let pat = &local.pat;
                replacements.retain(|(name, _)| !bindings(pat).iter().any(|id| id == name));
            }
        }
    }
    fn visit_item_mut(&mut self, _: &mut syn::Item) {}
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        let Ok(mut args) = expr::Arguments::parse(mac) else {
            return;
        };
        args.visit(self);
        self.changed |= args.expose_captures(self.replacements);
        mac.tokens = args.tokens();
    }
}
struct Class(bool);
impl VisitMut for Class {
    fn visit_expr_mut(&mut self, e: &mut Expr) {
        match e {
            Expr::Call(c) => {
                if !matches!(c.func.as_ref(), Expr::Path(p) if p.path.is_ident("old")) {
                    self.0 = true;
                }
            }
            Expr::MethodCall(c)
                if !matches!(
                    c.method.to_string().as_str(),
                    "len"
                        | "is_empty"
                        | "get"
                        | "contains"
                        | "is_some"
                        | "is_none"
                        | "is_ok"
                        | "is_err"
                        | "first"
                        | "last"
                        | "unwrap"
                        | "unwrap_or"
                        | "copied"
                        | "cloned"
                ) =>
            {
                self.0 = true
            }
            Expr::Closure(_) => self.0 = true,
            _ => (),
        }
        visit_mut::visit_expr_mut(self, e);
    }
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        if !expr::visit_macro(self, mac)
            || !matches!(attr_macro_name(mac).as_str(), "vec" | "matches")
        {
            self.0 = true;
        }
    }
}

type Old = (Ident, Expr, Clause);
fn extract_old(ts: TS, clause: &Clause, olds: &mut Vec<Old>) -> syn::Result<TS> {
    struct Olds<'a> {
        clause: &'a Clause,
        olds: &'a mut Vec<Old>,
        bound: Vec<Ident>,
        error: Option<syn::Error>,
    }
    impl Olds<'_> {
        fn condition(&mut self, expr: &mut Expr) {
            match expr {
                Expr::Let(l) => {
                    self.visit_expr_mut(&mut l.expr);
                    self.bound.extend(bindings(&l.pat));
                }
                Expr::Binary(b) if matches!(b.op, syn::BinOp::And(_)) => {
                    self.condition(&mut b.left);
                    self.condition(&mut b.right);
                }
                _ => self.visit_expr_mut(expr),
            }
        }
    }
    impl VisitMut for Olds<'_> {
        fn visit_item_mut(&mut self, _: &mut syn::Item) {}
        fn visit_arm_mut(&mut self, arm: &mut syn::Arm) {
            let old = self.bound.len();
            self.bound.extend(bindings(&arm.pat));
            if let Some((_, guard)) = &mut arm.guard {
                self.visit_expr_mut(guard);
            }
            self.visit_expr_mut(&mut arm.body);
            self.bound.truncate(old);
        }
        fn visit_block_mut(&mut self, block: &mut syn::Block) {
            let old = self.bound.len();
            for stmt in &mut block.stmts {
                self.visit_stmt_mut(stmt);
                if let Stmt::Local(local) = stmt {
                    self.bound.extend(bindings(&local.pat));
                }
            }
            self.bound.truncate(old);
        }
        fn visit_expr_for_loop_mut(&mut self, value: &mut syn::ExprForLoop) {
            self.visit_expr_mut(&mut value.expr);
            let old = self.bound.len();
            self.bound.extend(bindings(&value.pat));
            self.visit_block_mut(&mut value.body);
            self.bound.truncate(old);
        }
        fn visit_expr_while_mut(&mut self, value: &mut syn::ExprWhile) {
            let old = self.bound.len();
            self.condition(&mut value.cond);
            self.visit_block_mut(&mut value.body);
            self.bound.truncate(old);
        }
        fn visit_expr_if_mut(&mut self, value: &mut syn::ExprIf) {
            let old = self.bound.len();
            self.condition(&mut value.cond);
            self.visit_block_mut(&mut value.then_branch);
            self.bound.truncate(old);
            if let Some((_, body)) = &mut value.else_branch {
                self.visit_expr_mut(body);
            }
        }
        fn visit_expr_mut(&mut self, expr: &mut Expr) {
            if let Expr::Call(call) = expr {
                if matches!(call.func.as_ref(), Expr::Path(p) if p.path.is_ident("old")) {
                    if call.args.len() != 1 {
                        self.error =
                            Some(syn::Error::new_spanned(call, "old needs one expression"));
                        return;
                    }
                    let value = &call.args[0];
                    struct Nested(bool);
                    impl VisitMut for Nested {
                        fn visit_item_mut(&mut self, _: &mut syn::Item) {}
                        fn visit_expr_call_mut(&mut self, c: &mut syn::ExprCall) {
                            if matches!(c.func.as_ref(), Expr::Path(p) if p.path.is_ident("old")) {
                                self.0 = true;
                            }
                            visit_mut::visit_expr_call_mut(self, c);
                        }
                        fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
                            expr::visit_macro(self, mac);
                        }
                    }
                    let mut nested = Nested(false);
                    nested.visit_expr_mut(&mut value.clone());
                    let error = if nested.0 {
                        Some("nested old is not allowed")
                    } else if references(value, "result") {
                        Some("old cannot name result; it is evaluated at entry")
                    } else if self
                        .bound
                        .iter()
                        .any(|id| references(value, &id.to_string()))
                    {
                        Some(
                            "old cannot name a bound variable; snapshot the collection outside the binder",
                        )
                    } else {
                        None
                    };
                    if let Some(error) = error {
                        self.error = Some(syn::Error::new_spanned(value, error));
                        return;
                    }
                    let name = Ident::new(
                        &format!("__tide_old_{}", self.olds.len()),
                        Span::mixed_site(),
                    );
                    self.olds
                        .push((name.clone(), value.clone(), self.clause.clone()));
                    *expr = syn::parse_quote!((*#name.as_ref().unwrap()));
                    return;
                }
            }
            visit_mut::visit_expr_mut(self, expr);
        }
        fn visit_expr_closure_mut(&mut self, c: &mut syn::ExprClosure) {
            let old = self.bound.len();
            self.bound.extend(c.inputs.iter().flat_map(bindings));
            self.visit_expr_mut(&mut c.body);
            self.bound.truncate(old);
        }
        fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
            expr::visit_macro(self, mac);
        }
    }
    let mut value = expr::parse(ts)?;
    let mut visitor = Olds {
        clause,
        olds,
        bound: Vec::new(),
        error: None,
    };
    visitor.visit_expr_mut(&mut value);
    if let Some(error) = visitor.error {
        Err(error)
    } else {
        Ok(quote!(#value))
    }
}
fn lower(ts: TS) -> syn::Result<TS> {
    expr::parse(ts).map(|e| quote!(#e))
}

struct Loops<'a> {
    function: &'a str,
    olds: &'a mut Vec<Old>,
    error: Option<syn::Error>,
    index: usize,
    const_: bool,
    sampled: bool,
}
impl Loops<'_> {
    fn take(
        &mut self,
        body: &mut syn::Block,
        head: &Ident,
        range: bool,
    ) -> syn::Result<Option<TS>> {
        let mut checks = Vec::new();
        while let Some(Stmt::Macro(m)) = body.stmts.first() {
            if !m.mac.path.is_ident("invariant") {
                break;
            }
            let clause = Clause::new(
                "invariant",
                m.mac.tokens.clone(),
                m.mac.delimiter.span().join(),
                self.index,
            )?;
            self.index += 1;
            self.sampled |= clause.sampled;
            let expr = lower(extract_old(clause.tokens.clone(), &clause, self.olds)?)?;
            let schedule = if clause.sampled {
                quote!(::std::__contract::sampled_head(#head))
            } else if range {
                let r = Ident::new("__tide_range", Span::mixed_site());
                quote!(::core::cfg!(test) || #head == 1 || #r.start >= #r.end)
            } else {
                quote!(true)
            };
            checks.push(if self.const_ {
                const_fn::check(&clause, expr, self.function, schedule)
            } else {
                clause.check(expr, self.function, schedule)
            });
            body.stmts.remove(0);
        }
        Ok((!checks.is_empty()).then(|| quote!(#(#checks)*)))
    }
    fn rewrite(&mut self, e: &mut Expr) -> syn::Result<()> {
        let head = Ident::new("__tide_head", Span::mixed_site());
        let setup = quote!(let mut #head = 0u64;);
        let tick = quote!(#head = #head.saturating_add(1););
        let new = match e {
            Expr::ForLoop(fl) => {
                let range = matches!(expr::peel(&fl.expr), Expr::Range(r) if matches!(r.limits, RangeLimits::HalfOpen(_)) && r.start.is_some() && r.end.is_some());
                let Some(check) = self.take(&mut fl.body, &head, range)? else {
                    return Ok(());
                };
                let (label, pat, stmts, iter) = (&fl.label, &fl.pat, &fl.body.stmts, &fl.expr);
                let it = Ident::new("__tide_iter", Span::mixed_site());
                if range {
                    // Mixed-site ident in both the schedule and the loop avoids user bindings.
                    let r = Ident::new("__tide_range", Span::mixed_site());
                    quote!(match #iter { mut #r => { #setup #label loop {
                        #tick { let #pat = #r.start; #check }
                        match ::core::iter::Iterator::next(&mut #r) { ::core::option::Option::Some(#pat) => { #(#stmts)* }, ::core::option::Option::None => break }
                    } } })
                } else {
                    quote!(match ::core::iter::IntoIterator::into_iter(#iter) { mut #it => { #setup #label loop {
                        #tick #check
                        match ::core::iter::Iterator::next(&mut #it) { ::core::option::Option::Some(#pat) => { #(#stmts)* }, ::core::option::Option::None => break }
                    } } })
                }
            }
            Expr::While(w) => {
                let Some(check) = self.take(&mut w.body, &head, false)? else {
                    return Ok(());
                };
                let (label, body) = (&w.label, &w.body);
                let run = match w.cond.as_ref() {
                    Expr::Let(l) => {
                        let (p, x) = (&l.pat, &l.expr);
                        quote!(match #x { #p => #body, _ => break })
                    }
                    c => quote!(if #c #body else { break; }),
                };
                quote!({ #setup #label loop { #tick #check #run } })
            }
            Expr::Loop(l) => {
                let Some(check) = self.take(&mut l.body, &head, false)? else {
                    return Ok(());
                };
                let (label, body) = (&l.label, &l.body.stmts);
                quote!({ #setup #label loop { #tick #check #(#body)* } })
            }
            _ => return Ok(()),
        };
        *e = syn::parse2(new)?;
        Ok(())
    }
}
impl VisitMut for Loops<'_> {
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        expr::visit_macro(self, mac);
    }
    fn visit_item_mut(&mut self, _: &mut syn::Item) {}
    fn visit_expr_mut(&mut self, e: &mut Expr) {
        visit_mut::visit_expr_mut(self, e);
        if self.error.is_none() {
            if let Err(error) = self.rewrite(e) {
                self.error = Some(error);
            }
        }
    }
}

fn attr_macro_name(mac: &syn::Macro) -> String {
    mac.path.segments.last().unwrap().ident.to_string()
}
