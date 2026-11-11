//! Parse clause extensions into syn's expression tree before any semantic traversal.
use super::references;
use proc_macro2::{Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::quote;
use syn::{
    Expr, Pat, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
};

// `=` has implication's precedence and associativity. Tag only lexed `==>` operators;
// syn then owns argument, statement, initializer, closure and match-arm boundaries.
#[derive(PartialEq)]
struct Position(String, usize, usize);
fn position(span: Span) -> Position {
    let s = span.unwrap();
    Position(s.file(), s.line(), s.column())
}
#[derive(Default)]
struct Syntax {
    arrows: Vec<Position>,
    assignments: Vec<(Position, Span)>,
    quantifiers: Vec<(Position, bool)>,
    error: Option<syn::Error>,
}
pub(super) fn parse(tokens: TokenStream) -> syn::Result<Expr> {
    let mut syntax = Syntax::default();
    let mut expr = syn::parse2(syntax.lex(tokens))?;
    syntax.visit_expr_mut(&mut expr);
    if let Some((_, span)) = syntax
        .assignments
        .iter()
        .find(|(p, _)| syntax.arrows.contains(p))
    {
        return Err(syn::Error::new(
            *span,
            "ambiguous clause spans: `=` and `==>` share a source position",
        ));
    }
    if let Some(error) = syntax.error {
        Err(error)
    } else {
        Ok(expr)
    }
}
impl Syntax {
    fn lex(&mut self, tokens: TokenStream) -> TokenStream {
        let tokens: Vec<_> = tokens.into_iter().collect();
        let mut output = Vec::new();
        let mut i = 0;
        let punct =
            |i: usize, c| matches!(tokens.get(i), Some(TokenTree::Punct(p)) if p.as_char() == c);
        while i < tokens.len() {
            if punct(i, '=') && punct(i + 1, '=') && punct(i + 2, '>') {
                let mut op = Punct::new('=', Spacing::Alone);
                op.set_span(tokens[i].span());
                self.arrows.push(position(op.span()));
                output.push(TokenTree::Punct(op));
                i += 3;
                continue;
            }
            if let TokenTree::Punct(p) = &tokens[i] {
                if p.as_char() == '='
                    && p.spacing() == Spacing::Alone
                    && !matches!(i.checked_sub(1).and_then(|n| tokens.get(n)), Some(TokenTree::Punct(previous)) if previous.spacing() == Spacing::Joint)
                {
                    self.assignments.push((position(p.span()), p.span()));
                }
            }
            if let TokenTree::Ident(id) = &tokens[i] {
                if (id == "forall" || id == "exists") && punct(i + 1, '|') {
                    self.quantifiers
                        .push((position(tokens[i + 1].span()), id == "forall"));
                    i += 1;
                    continue;
                }
            }
            output.push(match &tokens[i] {
                TokenTree::Group(g)
                    if !(i >= 2
                        && punct(i - 1, '!')
                        && matches!(&tokens[i - 2], TokenTree::Ident(id) if !keyword(id))) =>
                {
                    let mut group = Group::new(g.delimiter(), self.lex(g.stream()));
                    group.set_span(g.span());
                    TokenTree::Group(group)
                }
                t => t.clone(),
            });
            i += 1;
        }
        output.into_iter().collect()
    }
    fn quantified(&mut self, closure: &syn::ExprClosure, forall: bool) -> syn::Result<Expr> {
        let fail = || {
            syn::Error::new(
                closure.span(),
                "FX0807: bounded quantifier needs `lo <= i && i < hi` (unsigned: `i < hi`) or `c.contains(&i)`",
            )
        };
        let Some(Pat::Type(pat)) = closure.inputs.first().filter(|_| closure.inputs.len() == 1)
        else {
            return Err(fail());
        };
        let Pat::Ident(binding) = pat.pat.as_ref() else {
            return Err(fail());
        };
        let (var, ty) = (&binding.ident, &pat.ty);
        let (guard, tail) = if forall {
            let Expr::Assign(a) = closure.body.as_ref() else {
                return Err(fail());
            };
            if !self.arrows.contains(&position(a.eq_token.span)) {
                return Err(fail());
            }
            (a.left.as_ref(), Some(a.right.as_ref()))
        } else {
            (closure.body.as_ref(), None)
        };
        let mut guards = Vec::new();
        conjuncts(guard, &mut guards);
        let mut at = 0;
        let lo = if let Some(Expr::Binary(b)) = guards.first().copied() {
            if matches!(b.op, syn::BinOp::Le(_))
                && is_var(&b.right, var)
                && !references(&b.left, &var.to_string())
            {
                at += 1;
                Some(*b.left.clone())
            } else {
                None
            }
        } else {
            None
        };
        let bound = *guards.get(at).ok_or_else(fail)?;
        at += 1;
        let mut pred: Expr = syn::parse_quote!(true);
        for guard in &guards[at..] {
            pred = syn::parse_quote!((#pred) && (#guard));
        }
        if let Some(tail) = tail {
            pred = syn::parse_quote!(!(#pred) || (#tail));
        }
        self.visit_expr_mut(&mut pred);
        let kind = if forall { "forall" } else { "exists" };
        let mut result = match bound {
            Expr::Binary(b)
                if matches!(b.op, syn::BinOp::Lt(_))
                    && is_var(&b.left, var)
                    && !references(&b.right, &var.to_string()) =>
            {
                let unsigned = matches!(ty.as_ref(), syn::Type::Path(t) if t.path.get_ident().is_some_and(|i| matches!(i.to_string().as_str(), "u8" | "u16" | "u32" | "u64" | "u128" | "usize")));
                let lo = lo
                    .or_else(|| unsigned.then(|| syn::parse_quote!(0)))
                    .ok_or_else(fail)?;
                let hi = &b.right;
                let helper = Ident::new(kind, Span::mixed_site());
                syn::parse_quote!(::std::__contract::#helper(#lo..#hi, |#var: #ty| #pred))
            }
            Expr::MethodCall(call)
                if lo.is_none()
                    && call.method == "contains"
                    && call.args.len() == 1
                    && matches!(&call.args[0], Expr::Reference(r) if r.mutability.is_none() && is_var(&r.expr, var))
                    && !references(&call.receiver, &var.to_string()) =>
            {
                let collection = &call.receiver;
                let helper = Ident::new(&format!("{kind}_in"), Span::mixed_site());
                syn::parse_quote!(::std::__contract::#helper((&#collection).into_iter(), |#var: #ty| #pred))
            }
            _ => return Err(fail()),
        };
        // Bounds can contain nested clause expressions too.
        if let Expr::Call(call) = &mut result {
            self.visit_expr_mut(&mut call.args[0]);
        }
        Ok(result)
    }
}
impl VisitMut for Syntax {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Closure(c) = expr {
            if let Some((_, forall)) = self
                .quantifiers
                .iter()
                .find(|(p, _)| *p == position(c.or1_token.span))
            {
                match self.quantified(c, *forall) {
                    Ok(value) => *expr = value,
                    Err(error) => self.error = Some(error),
                }
                return;
            }
        }
        visit_mut::visit_expr_mut(self, expr);
        if let Expr::Assign(a) = expr {
            if self.arrows.contains(&position(a.eq_token.span)) {
                let (left, right) = (&a.left, &a.right);
                *expr = syn::parse_quote!((!(#left) || (#right)));
            }
        }
    }
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        let original = mac.tokens.clone();
        mac.tokens = self.lex(original.clone());
        if let Ok(mut args) = Arguments::parse(mac) {
            // Normalize tagged operators before assignment-shaped arguments acquire meaning.
            match &mut args {
                Arguments::List(values, _) => {
                    for value in values {
                        self.visit_expr_mut(value);
                    }
                }
                _ => args.visit(self),
            }
            mac.tokens = args.tokens();
        } else {
            mac.tokens = original;
        }
    }
}
fn keyword(id: &Ident) -> bool {
    id == "gen" || syn::parse2::<Ident>(quote!(#id)).is_err()
}
fn is_var(expr: &Expr, var: &Ident) -> bool {
    matches!(expr, Expr::Path(p) if p.path.is_ident(var))
}
fn conjuncts<'a>(expr: &'a Expr, out: &mut Vec<&'a Expr>) {
    if let Expr::Binary(b) = expr {
        if matches!(b.op, syn::BinOp::And(_)) {
            conjuncts(&b.left, out);
            conjuncts(&b.right, out);
            return;
        }
    }
    out.push(expr);
}

pub(super) enum Arguments {
    List(Punctuated<Expr, Token![,]>, Option<usize>),
    Repeat(Expr, Expr),
    Matches(syn::ExprMatch),
}
impl Arguments {
    /// Expose only implicit captures of active inputs; other captures retain native scopes.
    pub(super) fn expose_captures(&mut self, replacements: &[(String, TokenStream)]) -> bool {
        use syn::ext::IdentExt;
        let Self::List(values, Some(index)) = self else {
            return false;
        };
        let Some(Expr::Lit(lit)) = values.iter().nth(*index) else {
            return false;
        };
        let syn::Lit::Str(string) = &lit.lit else {
            return false;
        };
        let text = string.value();
        let captures = format_captures(&text);
        let named: Vec<_> = values
            .iter()
            .skip(*index + 1)
            .filter_map(|arg| {
                let Expr::Assign(arg) = arg else {
                    return None;
                };
                let Expr::Path(path) = arg.left.as_ref() else {
                    return None;
                };
                path.path.get_ident().map(|id| id.unraw().to_string())
            })
            .collect();
        let mut used = quote!(#values).to_string();
        let mut exposed = Vec::new();
        for (name, value) in replacements {
            let name = name.strip_prefix("r#").unwrap_or(name);
            if named.iter().any(|n| n == name) || !captures.iter().any(|(_, n)| n == name) {
                continue;
            }
            let mut alias = format!("__tide_capture_{}", exposed.len());
            while used.contains(&alias) {
                alias.push('_');
            }
            used.push_str(&alias);
            exposed.push((
                name.to_owned(),
                Ident::new(&alias, Span::call_site()),
                value,
            ));
        }
        if exposed.is_empty() {
            return false;
        }
        let mut output = String::new();
        let mut end = 0;
        for (range, name) in captures {
            output.push_str(&text[end..range.start]);
            if let Some((_, alias, _)) = exposed.iter().find(|(n, _, _)| *n == name) {
                output.push_str(&alias.to_string());
            } else {
                output.push_str(&text[range.clone()]);
            }
            end = range.end;
        }
        output.push_str(&text[end..]);
        let span = string.span();
        let string = syn::LitStr::new(&output, span);
        *values.iter_mut().nth(*index).unwrap() = syn::parse_quote!(#string);
        for (_, alias, value) in exposed {
            values.push(syn::parse_quote!(#alias = #value));
        }
        true
    }
    pub(super) fn parse(mac: &syn::Macro) -> syn::Result<Self> {
        let name = mac.path.segments.last().unwrap().ident.to_string();
        if name == "matches" {
            return syn::parse2::<Matches>(mac.tokens.clone()).map(|m| Self::Matches(m.0));
        }
        if name == "vec" {
            if let Ok(repeat) = syn::parse2::<Repeat>(mac.tokens.clone()) {
                return Ok(Self::Repeat(repeat.0, repeat.1));
            }
        }
        if !matches!(
            name.as_str(),
            "vec"
                | "format"
                | "format_args"
                | "print"
                | "println"
                | "eprint"
                | "eprintln"
                | "write"
                | "writeln"
                | "panic"
                | "assert"
                | "assert_eq"
                | "assert_ne"
                | "debug_assert"
                | "debug_assert_eq"
                | "debug_assert_ne"
                | "dbg"
                | "todo"
                | "unimplemented"
                | "unreachable"
        ) {
            return Err(syn::Error::new(
                mac.path.span(),
                "builtin has no expression argument grammar",
            ));
        }
        use syn::parse::Parser;
        let format = match name.as_str() {
            "format" | "format_args" | "print" | "println" | "eprint" | "eprintln" | "panic"
            | "todo" | "unimplemented" | "unreachable" => Some(0),
            "write" | "writeln" | "assert" | "debug_assert" => Some(1),
            "assert_eq" | "assert_ne" | "debug_assert_eq" | "debug_assert_ne" => Some(2),
            _ => None,
        };
        Punctuated::<Expr, Token![,]>::parse_terminated
            .parse2(mac.tokens.clone())
            .map(|args| Self::List(args, format))
    }
    pub(super) fn visit(&mut self, visitor: &mut impl VisitMut) {
        match self {
            Self::List(args, format) => {
                for (i, arg) in args.iter_mut().enumerate() {
                    if format.is_some_and(|format| i > format) {
                        if let Expr::Assign(named) = arg {
                            if matches!(named.left.as_ref(), Expr::Path(p) if p.path.get_ident().is_some())
                            {
                                visitor.visit_expr_mut(&mut named.right);
                                continue;
                            }
                        }
                    }
                    visitor.visit_expr_mut(arg);
                }
            }
            Self::Repeat(value, count) => {
                visitor.visit_expr_mut(value);
                visitor.visit_expr_mut(count);
            }
            Self::Matches(m) => visitor.visit_expr_match_mut(m),
        }
    }
    pub(super) fn tokens(&self) -> TokenStream {
        match self {
            Self::List(args, _) => quote!(#args),
            Self::Repeat(value, count) => quote!(#value; #count),
            Self::Matches(m) => {
                let value = &m.expr;
                let arm = &m.arms[0];
                let pat = &arm.pat;
                let guard = arm.guard.as_ref().map(|(_, e)| quote!(if #e));
                quote!(#value, #pat #guard)
            }
        }
    }
}

fn format_captures(text: &str) -> Vec<(std::ops::Range<usize>, String)> {
    use syn::{ext::IdentExt, parse::Parser};
    let ident = |s: &str| {
        if s.starts_with(char::is_whitespace) || s.contains('#') {
            return None;
        }
        Ident::parse_any
            .parse_str(s)
            .ok()
            .map(|id| id.unraw().to_string())
    };
    let mut captures = Vec::new();
    let mut at = 0;
    while at < text.len() {
        if text[at..].starts_with("{{") {
            at += 2;
            continue;
        }
        if !text[at..].starts_with('{') {
            at += text[at..].chars().next().unwrap().len_utf8();
            continue;
        }
        let start = at + 1;
        let Some(close) = text[start..].find('}').map(|i| start + i) else {
            break;
        };
        let separator = text[start..close].find(':').map_or(close, |i| start + i);
        let argument = &text[start..separator];
        if let Some(name) = ident(argument) {
            captures.push((start..start + argument.trim_end().len(), name));
        }
        if separator < close {
            let mut word = None;
            for (offset, c) in text[separator + 1..close].char_indices() {
                let pos = separator + 1 + offset;
                if c.is_alphanumeric() || c == '_' || !c.is_ascii() {
                    word.get_or_insert(pos);
                } else {
                    if c == '$' {
                        if let Some(begin) = word {
                            if let Some(name) = ident(&text[begin..pos]) {
                                captures.push((begin..pos, name));
                            }
                        }
                    }
                    word = None;
                }
            }
        }
        at = close + 1;
    }
    captures
}
struct Repeat(Expr, Expr);
impl Parse for Repeat {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let value = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(Self(value, input.parse()?))
    }
}
struct Matches(syn::ExprMatch);
impl Parse for Matches {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let expr: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let pat = Pat::parse_multi_with_leading_vert(input)?;
        let guard = if input.peek(Token![if]) {
            let token = input.parse()?;
            let expr: Expr = input.parse()?;
            Some((token, Box::new(expr)))
        } else {
            None
        };
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        let mut arm: syn::Arm = syn::parse_quote!(#pat => true);
        arm.guard = guard;
        let mut result: syn::ExprMatch = syn::parse_quote!(match #expr { _ => false });
        result.arms.insert(0, arm);
        Ok(Self(result))
    }
}
/// Unknown or malformed macro grammars stay untouched, never token-rewritten.
pub(super) fn visit_macro(visitor: &mut impl VisitMut, mac: &mut syn::Macro) -> bool {
    let Ok(mut args) = Arguments::parse(mac) else {
        return false;
    };
    args.visit(visitor);
    mac.tokens = args.tokens();
    true
}

pub(super) fn peel(expr: &Expr) -> &Expr {
    match expr {
        Expr::Paren(e) => peel(&e.expr),
        Expr::Group(e) => peel(&e.expr),
        e => e,
    }
}
pub(super) fn peel_type(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Paren(t) => peel_type(&t.elem),
        syn::Type::Group(t) => peel_type(&t.elem),
        t => t,
    }
}
