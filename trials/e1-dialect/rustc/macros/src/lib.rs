//! Oracle-only shims for the toolchain attributes the D18 spike's macros do
//! not cover. They exist so rustc can type- and borrow-check the trial's
//! programs; they do not implement Tide's semantics.
//!
//! - `#[test]` replaces the builtin through the stub prelude. A test with no
//!   parameters becomes a real `#[test]`; a property test (one with
//!   parameters, reference 10.2) becomes a plain fn, so its body is still
//!   checked but nothing generates inputs.
//! - `#[sim_test(..)]` likewise becomes a plain fn taking its `SimRoot`.
//! - `derive(Error)` follows thiserror's surface (reference 2.2): `Display`
//!   from `#[error("..")]` (positional `{0}` fields included) or
//!   `#[error(transparent)]`, `Error::source` from `#[source]`, `#[from]`, or
//!   a field named `source`, and `From` for `#[from]` fields.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TS;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Fields, ItemFn, LitStr, parse_macro_input};

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let f = parse_macro_input!(item as ItemFn);
    if f.sig.inputs.is_empty() {
        quote!(#[::core::prelude::v1::test] #f).into()
    } else {
        quote!(#[allow(dead_code)] #f).into()
    }
}

#[proc_macro_attribute]
pub fn sim_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let f = parse_macro_input!(item as ItemFn);
    quote!(#[allow(dead_code)] #f).into()
}

enum Msg {
    Format(String),
    Transparent,
}

fn message(attrs: &[Attribute]) -> syn::Result<Option<Msg>> {
    let Some(a) = attrs.iter().find(|a| a.path().is_ident("error")) else {
        return Ok(None);
    };
    let ts: TS = a.meta.require_list()?.tokens.clone();
    if ts.to_string() == "transparent" {
        return Ok(Some(Msg::Transparent));
    }
    let lit: LitStr = syn::parse2(ts)?;
    Ok(Some(Msg::Format(positional(&lit.value()))))
}

/// thiserror's `{0}` names field 0; the generated match binds it as `_0`.
fn positional(s: &str) -> String {
    let (mut out, mut chars) = (String::new(), s.chars().peekable());
    while let Some(c) = chars.next() {
        out.push(c);
        if c == '{' {
            if chars.peek() == Some(&'{') {
                out.push(chars.next().unwrap());
            } else if chars.peek().is_some_and(|d| d.is_ascii_digit()) {
                out.push('_');
            }
        }
    }
    out
}

fn has(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(name))
}

/// The pattern binding every field, the `source()` arm body, and a `From` impl.
fn arm(path: TS, fields: &Fields, ty: &syn::Ident) -> (TS, Option<TS>, Option<TS>) {
    let mut binds = Vec::new();
    let (mut source, mut from) = (None, None);
    for (i, f) in fields.iter().enumerate() {
        let id = f.ident.clone().unwrap_or_else(|| format_ident!("_{}", i));
        let is_from = has(&f.attrs, "from");
        if is_from || has(&f.attrs, "source") || f.ident.as_ref().is_some_and(|n| n == "source") {
            source = Some(quote!(::core::option::Option::Some(#id as &(dyn ::std::error::Error + 'static))));
        }
        if is_from {
            let fty = &f.ty;
            let build = match &f.ident {
                Some(n) => quote!(#path { #n: v }),
                None => quote!(#path(v)),
            };
            from = Some(quote!(impl ::core::convert::From<#fty> for #ty {
                fn from(v: #fty) -> Self { #build }
            }));
        }
        binds.push(id);
    }
    let pat = match fields {
        Fields::Named(_) => quote!(#path { #(#binds),* }),
        Fields::Unnamed(_) => quote!(#path( #(#binds),* )),
        Fields::Unit => quote!(#path),
    };
    (pat, source, from)
}

fn display(msg: Option<Msg>, fields: &Fields) -> syn::Result<TS> {
    match msg {
        Some(Msg::Format(s)) => Ok(quote!(::core::write!(__f, #s))),
        Some(Msg::Transparent) => {
            let id = fields.iter().next().and_then(|f| f.ident.clone()).unwrap_or_else(|| format_ident!("_0"));
            Ok(quote!(::core::fmt::Display::fmt(#id, __f)))
        }
        None => Err(syn::Error::new(proc_macro2::Span::call_site(), "missing #[error(..)]")),
    }
}

#[proc_macro_derive(Error, attributes(error, from, source))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn derive(input: &DeriveInput) -> syn::Result<TS> {
    let ty = &input.ident;
    let cases: Vec<(TS, &Fields, &[Attribute])> = match &input.data {
        Data::Struct(s) => vec![(quote!(#ty), &s.fields, &input.attrs)],
        Data::Enum(e) => e
            .variants
            .iter()
            .map(|v| {
                let vid = &v.ident;
                (quote!(#ty::#vid), &v.fields, &v.attrs[..])
            })
            .collect(),
        Data::Union(_) => return Err(syn::Error::new_spanned(ty, "derive(Error) on a union")),
    };
    let (mut disp, mut src, mut froms) = (Vec::new(), Vec::new(), Vec::new());
    for (path, fields, attrs) in cases {
        let (pat, source, from) = arm(path, fields, ty);
        let body = display(message(attrs)?, fields)?;
        disp.push(quote!(#pat => #body,));
        if let Some(b) = source {
            src.push(quote!(#pat => #b,));
        }
        froms.extend(from);
    }
    Ok(quote! {
        #[allow(unused_variables)]
        impl ::core::fmt::Display for #ty {
            fn fmt(&self, __f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self { #(#disp)* }
            }
        }
        #[allow(unused_variables, unreachable_patterns)]
        impl ::std::error::Error for #ty {
            fn source(&self) -> ::core::option::Option<&(dyn ::std::error::Error + 'static)> {
                match self { #(#src)* _ => ::core::option::Option::None }
            }
        }
        #(#froms)*
    })
}
