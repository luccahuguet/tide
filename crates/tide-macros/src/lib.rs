//! Tide's toolchain macros, which Tide's std re-exports: `#[tide::main]` (`std::tide`),
//! `#[test]`, `#[pure]`, executable contracts (`requires`, `ensures`, `invariants`, `invariant!`),
//! the sink and panic macros (`print!` and its kin, `panic!`), and the `Error`, `Encode`, and `Decode`
//! derives. The admission driver
//! recognizes `#[tide::main]` as the macro named `main` that Tide's std exports, and treats the
//! entry point it generates as the toolchain's own code only when it is the crate-root `main` of
//! an executable crate.

mod codec;
mod contract;
mod error;
mod sim_test;
mod test;
mod test_guidance;

use proc_macro::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};

/// Asserts authority-freedom of a function, checked by the driver's existing FX0511 rule.
/// Arguments and non-function targets are errors. Original tokens retain their spans.
/// A Rust 2024 `safe fn` foreign declaration gets this macro's generic target error rather
/// than FX0511 and is still rejected.
#[proc_macro_attribute]
pub fn pure(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Some(t) = attr.into_iter().next() {
        return diagnostic(t.span(), "`#[pure]` takes no arguments");
    }
    if syn::parse::<syn::ItemFn>(item.clone()).is_err()
        && syn::parse::<syn::TraitItemFn>(item.clone()).is_err()
        && syn::parse::<syn::ForeignItemFn>(item.clone()).is_err()
    {
        return diagnostic(Span::call_site(), "`#[pure]` goes on a function");
    }
    item
}

/// Derives Display, Error, and field conversions (reference 2.2).
#[proc_macro_derive(Error, attributes(error, from, source))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    error::expand(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Keeps ordinary libtest entries and lowers parameterized tests to Tide's property runner.
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    test::expand(attr.into(), syn::parse_macro_input!(item as syn::ItemFn))
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Runs a simulation test over 100 seeds, or `seeds = N`, in a child process.
#[proc_macro_attribute]
pub fn sim_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    sim_test::expand(attr.into(), syn::parse_macro_input!(item as syn::ItemFn))
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derives JSON encoding, read by `std::json::encode` (reference 2.2).
#[proc_macro_derive(Encode, attributes(codec))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    codec::encode(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Derives JSON decoding, read by `std::json::decode` (reference 2.2).
#[proc_macro_derive(Decode, attributes(codec))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    codec::decode(input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// `#[tide::main] fn main(root: Root) -> R` (03 ROOT-1, reference 6.2): generates Rust's
/// parameterless `main`, which calls Tide's runtime entry with the user's `main`. The entry mints
/// the process's one `Root`, passes it when `main` takes a parameter, and reports what `main`
/// returns as Rust's entry point does.
#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Some(t) = attr.into_iter().next() {
        return diagnostic(t.span(), "`#[tide::main]` takes no arguments");
    }
    let mut tokens: Vec<TokenTree> = item.into_iter().collect();
    // The name follows the first top-level `fn`; attributes are bracketed groups.
    let found = tokens
        .windows(2)
        .position(
            |w| matches!(w, [TokenTree::Ident(f), TokenTree::Ident(_)] if f.to_string() == "fn"),
        )
        .map(|i| i + 1);
    let Some(name) = found.filter(|&i| tokens[i].to_string() == "main") else {
        let span = found.map_or(Span::call_site(), |i| tokens[i].span());
        return diagnostic(span, "`#[tide::main]` goes on the binary's `fn main`");
    };
    let takes_root = tokens[name..].iter().find_map(|t| match t {
        TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => {
            Some(!g.stream().is_empty())
        }
        _ => None,
    });
    tokens[name] = TokenTree::Ident(Ident::new("__tide_main", tokens[name].span()));
    let call = if takes_root == Some(true) {
        "::std::__rt::enter(__tide_main)"
    } else {
        "::std::__rt::enter(|_| __tide_main())"
    };
    // Tokens parsed here carry the call site's span, so the generated `main` and its call are
    // the attribute's expansion; the user's function keeps its own spans.
    let mut body: TokenStream = tokens.into_iter().collect();
    body.extend(call.parse::<TokenStream>().unwrap());
    let mut out: TokenStream = "fn main() -> ::std::process::ExitCode".parse().unwrap();
    out.extend([TokenTree::Group(Group::new(Delimiter::Brace, body))]);
    out
}

/// `print!(..)`: formats a record and writes it to the `out` sink (reference 6.5).
#[proc_macro]
pub fn print(input: TokenStream) -> TokenStream {
    sink("out", input, false)
}

/// `println!(..)`: `print!` with a newline.
#[proc_macro]
pub fn println(input: TokenStream) -> TokenStream {
    sink("out", input, true)
}

/// `eprint!(..)`: formats a record and writes it to the `err` sink.
#[proc_macro]
pub fn eprint(input: TokenStream) -> TokenStream {
    sink("err", input, false)
}

/// `eprintln!(..)`: `eprint!` with a newline.
#[proc_macro]
pub fn eprintln(input: TokenStream) -> TokenStream {
    sink("err", input, true)
}

/// Prints Rust-style source locations and debug values to stderr, returning the values.
/// The admission driver rejects this macro in release builds (SINK-8).
#[proc_macro]
pub fn dbg(input: TokenStream) -> TokenStream {
    use syn::parse::Parser;
    let values = match syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
        .parse(input)
    {
        Ok(values) => values,
        Err(error) => return error.into_compile_error().into(),
    };
    if values.is_empty() {
        return quote::quote! {
            ::std::__sink::dbg(::core::format_args!("[{}:{}:{}]",
                ::core::file!(), ::core::line!(), ::core::column!()), true)
        }
        .into();
    }
    let value = proc_macro2::Ident::new("__tide_dbg_value", proc_macro2::Span::mixed_site());
    let mut expressions = values.iter().map(|expr| {
        quote::quote! {
            // Match keeps Rust dbg!'s temporary lifetimes and evaluates the argument once.
            match (#expr) {
                #value => {
                    ::std::__sink::dbg(::core::format_args!("[{}:{}:{}] {} = {:#?}",
                        ::core::file!(), ::core::line!(), ::core::column!(),
                        ::core::stringify!(#expr), &&#value as &dyn ::std::fmt::Debug), true);
                    #value
                }
            }
        }
    });
    if values.len() == 1 {
        expressions.next().unwrap().into()
    } else {
        quote::quote! { (#(#expressions,)*) }.into()
    }
}

/// Writes an error-level log record, formatting even when disabled.
#[proc_macro]
pub fn error(input: TokenStream) -> TokenStream { log_sink("Error", input) }

/// Writes a warn-level log record, formatting even when disabled.
#[proc_macro]
pub fn warn(input: TokenStream) -> TokenStream { log_sink("Warn", input) }

/// Writes an info-level log record, formatting even when disabled.
#[proc_macro]
pub fn info(input: TokenStream) -> TokenStream { log_sink("Info", input) }

/// Writes a debug-level log record, formatting even when disabled.
#[proc_macro]
pub fn debug(input: TokenStream) -> TokenStream { log_sink("Debug", input) }

/// Writes a trace-level log record, formatting even when disabled.
#[proc_macro]
pub fn trace(input: TokenStream) -> TokenStream { log_sink("Trace", input) }

fn log_sink(level: &str, input: TokenStream) -> TokenStream {
    let mut args: TokenStream = format!("::std::log::Level::{level}, ::core::format_args!").parse().unwrap();
    args.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, input))]);
    let mut call: TokenStream = "::std::__sink::log".parse().unwrap();
    call.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, args))]);
    call
}

/// `panic!(..)`, as `core`'s.
#[proc_macro]
pub fn panic(input: TokenStream) -> TokenStream {
    let mut out: TokenStream = "::core::panic!".parse().unwrap();
    out.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, input))]);
    out
}

/// `::std::__sink::<name>(::core::format_args!(<input>), <newline>)`. The input keeps its
/// spans, so format strings capture the caller's variables. As in Rust, only the line forms take
/// no arguments.
fn sink(name: &str, input: TokenStream, newline: bool) -> TokenStream {
    let input = if input.is_empty() && newline {
        "\"\"".parse().unwrap()
    } else {
        input
    };
    let mut args: TokenStream = "::core::format_args!".parse().unwrap();
    args.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, input))]);
    args.extend(format!(", {newline}").parse::<TokenStream>().unwrap());
    let mut call: TokenStream = format!("::std::__sink::{name}").parse().unwrap();
    call.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, args))]);
    call
}

/// `compile_error!(message)` at `span`.
fn diagnostic(span: Span, message: &str) -> TokenStream {
    let tokens: TokenStream = format!("::core::compile_error!({message:?});")
        .parse()
        .unwrap();
    tokens
        .into_iter()
        .map(|mut t| {
            t.set_span(span);
            t
        })
        .collect()
}

/// Checks a precondition at entry, before any snapshots; const fns use literal checks.
#[proc_macro_attribute]
pub fn requires(attr: TokenStream, item: TokenStream) -> TokenStream {
    contract::expand("requires", attr.into(), item.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
/// Checks a postcondition at every normal return (support/limits: tide-std crate docs).
#[proc_macro_attribute]
pub fn ensures(attr: TokenStream, item: TokenStream) -> TokenStream {
    contract::expand("ensures", attr.into(), item.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
/// Rewrites loop invariants without adding function-boundary clauses.
#[proc_macro_attribute]
pub fn invariants(attr: TokenStream, item: TokenStream) -> TokenStream {
    contract::expand("invariants", attr.into(), item.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
/// Reports an invariant that no enclosing contract attribute rewrote.
#[proc_macro]
pub fn invariant(input: TokenStream) -> TokenStream {
    diagnostic(
        input
            .into_iter()
            .next()
            .map_or(Span::call_site(), |t| t.span()),
        "FX0806: invariant!(..) must open a loop body in a function with #[invariants], #[requires], or #[ensures]",
    )
}
