//! Syntactic guidance only; the original requires always checks the resulting input.
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::visit_mut::{self, VisitMut};
use syn::{BinOp, Expr, FnArg, GenericArgument, ItemFn, Pat, PathArguments, Type};

#[derive(Default)]
struct Plan {
    lower: Vec<(Expr, bool)>,
    upper: Vec<(Expr, bool)>,
    fixed: Vec<Expr>,
    sorted: bool,
    some: bool,
    contains: Vec<Expr>,
    dependencies: Vec<usize>,
}

pub struct Guidance {
    names: Vec<Option<Ident>>,
    plans: Vec<Plan>,
    pub order: Vec<usize>,
}

fn peel(expr: &Expr) -> &Expr {
    match expr {
        Expr::Paren(p) => peel(&p.expr),
        Expr::Group(p) => peel(&p.expr),
        _ => expr,
    }
}
fn parameter(expr: &Expr, names: &[Option<Ident>]) -> Option<usize> {
    let Expr::Path(p) = peel(expr) else {
        return None;
    };
    names
        .iter()
        .position(|name| name.as_ref().is_some_and(|name| p.path.is_ident(name)))
}
fn length(expr: &Expr, names: &[Option<Ident>]) -> Option<usize> {
    let Expr::MethodCall(call) = peel(expr) else {
        return None;
    };
    (call.method == "len" && call.args.is_empty())
        .then(|| parameter(&call.receiver, names))
        .flatten()
}
fn type_name(ty: &Type) -> Option<&syn::PathSegment> {
    match ty {
        Type::Reference(r) => type_name(&r.elem),
        Type::Paren(p) => type_name(&p.elem),
        Type::Group(g) => type_name(&g.elem),
        Type::Path(p) => p.path.segments.last(),
        _ => None,
    }
}
fn scalar(ty: &Type) -> bool {
    type_name(ty).is_some_and(|s| {
        matches!(
            s.ident.to_string().as_str(),
            "u8" | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "f32"
                | "f64"
        )
    })
}
fn sequence(ty: &Type) -> bool {
    match ty {
        Type::Reference(r) => sequence(&r.elem),
        Type::Slice(_) => true,
        _ => type_name(ty)
            .is_some_and(|s| matches!(s.ident.to_string().as_str(), "Vec" | "String" | "str")),
    }
}
fn vector(ty: &Type) -> bool {
    match ty {
        Type::Reference(r) => vector(&r.elem),
        Type::Slice(_) => true,
        _ => type_name(ty).is_some_and(|s| s.ident == "Vec"),
    }
}

// Bounds and inserted elements use owned values; method receivers keep their borrows.
fn owned_value(
    expr: &Expr,
    parameters: &[&syn::PatType],
    names: &[Option<Ident>],
) -> Option<(Expr, Option<usize>)> {
    match peel(expr) {
        Expr::Reference(value) => owned_value(&value.expr, parameters, names),
        Expr::Path(_) => {
            let index = parameter(expr, names)?;
            let mut ty = parameters[index].ty.as_ref();
            if !scalar(ty)
                && !type_name(ty).is_some_and(|s| {
                    matches!(s.ident.to_string().as_str(), "char" | "bool" | "String")
                })
            {
                return None;
            }
            let mut value = expr.clone();
            loop {
                match ty {
                    Type::Reference(reference) => {
                        value = syn::parse_quote!(*(#value));
                        ty = &reference.elem;
                    }
                    Type::Paren(p) => ty = &p.elem,
                    Type::Group(g) => ty = &g.elem,
                    _ => break,
                }
            }
            Some((value, Some(index)))
        }
        Expr::Lit(_) => Some((expr.clone(), None)),
        Expr::Unary(u)
            if matches!(u.op, syn::UnOp::Neg(_)) && matches!(peel(&u.expr), Expr::Lit(_)) =>
        {
            Some((expr.clone(), None))
        }
        _ if length(expr, names).is_some_and(|i| sequence(&parameters[i].ty)) => {
            Some((expr.clone(), length(expr, names)))
        }
        _ => None,
    }
}

impl Guidance {
    pub fn new(body: &ItemFn) -> syn::Result<Self> {
        let parameters: Vec<_> = body
            .sig
            .inputs
            .iter()
            .filter_map(|p| match p {
                FnArg::Typed(p) => Some(p),
                _ => None,
            })
            .collect();
        let names: Vec<_> = parameters
            .iter()
            .map(|p| match p.pat.as_ref() {
                Pat::Ident(p) if p.subpat.is_none() => Some(p.ident.clone()),
                _ => None,
            })
            .collect();
        let mut plans: Vec<_> = (0..parameters.len()).map(|_| Plan::default()).collect();
        let mut conjuncts = Vec::new();
        for attr in &body.attrs {
            if attr
                .path()
                .segments
                .last()
                .is_some_and(|s| s.ident == "requires")
            {
                let expr: Expr = syn::parse2(crate::contract::lower(
                    attr.meta.require_list()?.tokens.clone(),
                )?)?;
                let mut parts = Vec::new();
                crate::contract::requires_conjuncts(&expr, &mut parts);
                conjuncts.extend(parts.into_iter().cloned());
            }
        }
        for expr in &conjuncts {
            if let Expr::MethodCall(call) = peel(expr) {
                if let Some(index) = parameter(&call.receiver, &names) {
                    let plan = &mut plans[index];
                    match call.method.to_string().as_str() {
                        "is_sorted" if call.args.is_empty() && vector(&parameters[index].ty) => {
                            plan.sorted = true
                        }
                        "is_some"
                            if call.args.is_empty()
                                && type_name(&parameters[index].ty)
                                    .is_some_and(|s| s.ident == "Option") =>
                        {
                            plan.some = true
                        }
                        "contains" if call.args.len() == 1 && vector(&parameters[index].ty) => {
                            if let Expr::Reference(value) = peel(&call.args[0]) {
                                // Moving an arbitrary element twice would add an unsupported Clone bound.
                                if let Some((owned, dependency)) =
                                    owned_value(&value.expr, &parameters, &names)
                                {
                                    plan.contains.push(owned);
                                    plan.lower.push((syn::parse_quote!(1usize), true));
                                    if let Some(i) = dependency {
                                        plan.dependencies.push(i);
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
                continue;
            }
            let Expr::Binary(binary) = peel(expr) else {
                continue;
            };
            let (left, right, inclusive, equal) = match binary.op {
                BinOp::Lt(_) => (&*binary.left, &*binary.right, false, false),
                BinOp::Le(_) => (&*binary.left, &*binary.right, true, false),
                BinOp::Gt(_) => (&*binary.right, &*binary.left, false, false),
                BinOp::Ge(_) => (&*binary.right, &*binary.left, true, false),
                BinOp::Eq(_) => (&*binary.left, &*binary.right, true, true),
                _ => continue,
            };
            // A length source precedes its index, regardless of signature order.
            if !equal
                && let (Some(i), Some(xs)) = (parameter(left, &names), length(right, &names))
                && scalar(&parameters[i].ty)
                && sequence(&parameters[xs].ty)
                && i != xs
            {
                if !inclusive {
                    plans[xs].lower.push((syn::parse_quote!(1usize), true));
                }
                plans[i].upper.push((right.clone(), inclusive));
                plans[i].dependencies.push(xs);
                continue;
            }
            // A parameter relation draws the upper endpoint after the lower one.
            let target =
                if let (Some(lo), Some(hi)) = (parameter(left, &names), parameter(right, &names)) {
                    if lo == hi {
                        continue;
                    }
                    Some((hi, false, left, false))
                } else if let Some(i) = parameter(left, &names) {
                    Some((i, true, right, false))
                } else if let Some(i) = parameter(right, &names) {
                    Some((i, false, left, false))
                } else if let Some(i) = length(left, &names) {
                    Some((i, true, right, true))
                } else {
                    length(right, &names).map(|i| (i, false, left, true))
                };
            let Some((index, upper, value, len)) = target else {
                continue;
            };
            let ty = &parameters[index].ty;
            if !(if len {
                sequence(ty)
            } else {
                scalar(ty)
                    || (equal
                        && type_name(ty).is_some_and(|s| {
                            matches!(s.ident.to_string().as_str(), "char" | "bool")
                        }))
            }) {
                continue;
            }
            let Some((value, dependency)) = owned_value(value, &parameters, &names) else {
                continue;
            };
            if dependency == Some(index) {
                continue;
            }
            let plan = &mut plans[index];
            if let Some(i) = dependency {
                plan.dependencies.push(i);
            }
            if equal && !len && !scalar(ty) {
                plan.fixed.push(value.clone());
            } else {
                if !upper || equal {
                    plan.lower.push((value.clone(), inclusive));
                }
                if upper || equal {
                    plan.upper.push((value.clone(), inclusive));
                }
            }
        }
        let mut order = Vec::new();
        while order.len() < plans.len() {
            let next = (0..plans.len()).find(|i| {
                !order.contains(i) && plans[*i].dependencies.iter().all(|d| order.contains(d))
            });
            if let Some(next) = next {
                order.push(next);
            } else {
                // Cyclic dependencies cannot guide a draw; the clauses still filter.
                plans = (0..parameters.len()).map(|_| Plan::default()).collect();
                order = (0..parameters.len()).collect();
                break;
            }
        }
        Ok(Self {
            names,
            plans,
            order,
        })
    }

    fn expression(&self, expr: &Expr) -> Expr {
        struct Arguments<'a>(&'a [Option<Ident>]);
        impl VisitMut for Arguments<'_> {
            fn visit_expr_path_mut(&mut self, expr: &mut syn::ExprPath) {
                if let Some(index) = self
                    .0
                    .iter()
                    .position(|name| name.as_ref().is_some_and(|name| expr.path.is_ident(name)))
                {
                    let arg =
                        format_ident!("__tide_arg_{index}", span = proc_macro2::Span::mixed_site());
                    *expr = syn::parse_quote!(#arg);
                } else {
                    visit_mut::visit_expr_path_mut(self, expr);
                }
            }
        }
        let mut expr = expr.clone();
        Arguments(&self.names).visit_expr_mut(&mut expr);
        expr
    }

    pub fn draw(
        &self,
        index: usize,
        ty: &Type,
        owned: TokenStream,
        g: &Ident,
        target: &Ident,
        mutable: bool,
    ) -> TokenStream {
        let plan = &self.plans[index];
        let lower = plan.lower.iter().map(|(v, i)| {
            let v = self.expression(v);
            quote!((#v, #i))
        });
        let upper = plan.upper.iter().map(|(v, i)| {
            let v = self.expression(v);
            quote!((#v, #i))
        });
        let ordinary = quote!(<#owned as ::std::test::Arbitrary>::arbitrary(#g));
        let initial = if !plan.lower.is_empty() || !plan.upper.is_empty() {
            if sequence(ty) {
                quote!(::std::test::sequence::<#owned>(#g, &[#(#lower),*], &[#(#upper),*]))
            } else {
                quote!(::std::test::range::<#owned>(#g, &[#(#lower),*], &[#(#upper),*]))
            }
        } else if let Some(first) = plan.fixed.first() {
            let first = self.expression(first);
            let fixed = plan.fixed.iter().map(|v| self.expression(v));
            quote!(if true #(&& (#first == #fixed))* { #first } else { #ordinary })
        } else if plan.some {
            let segment = type_name(ty).unwrap();
            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return ordinary;
            };
            let Some(GenericArgument::Type(inner)) = arguments.args.first() else {
                return ordinary;
            };
            quote!(::core::option::Option::Some(<#inner as ::std::test::Arbitrary>::arbitrary(#g)))
        } else {
            ordinary
        };
        let insert = plan.contains.iter().map(|value| {
            let value = self.expression(value);
            quote! {
                if !#target.is_empty() && !#target.contains(&(#value)) {
                    let index = #g.index(#target.len());
                    #target[index] = (#value).clone();
                }
            }
        });
        let sort = plan.sorted.then(|| quote!(#target.sort_by(|a, b| a.partial_cmp(b).unwrap_or(::core::cmp::Ordering::Equal));));
        let mutable = (mutable || plan.sorted || !plan.contains.is_empty()).then(|| quote!(mut));
        quote! {
            let #mutable #target: #owned = #initial;
            #(#insert)*
            #sort
        }
    }
}
