//! Tide's toolchain macros, which Tide's std re-exports: `#[tide::main]` (`std::tide`), and the
//! sink and panic macros (`print!` and its kin, `panic!`). The admission driver
//! recognizes `#[tide::main]` as the macro named `main` that Tide's std exports, and treats the
//! entry point it generates as the toolchain's own code only when it is the crate-root `main` of
//! an executable crate.

use proc_macro::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};

/// `#[tide::main] fn main(root: Root) -> R` (03 ROOT-1, reference 6.2): generates Rust's
/// parameterless `main`, which calls Tide's runtime entry with the user's `main`. The entry mints
/// the process's one `Root`, passes it when `main` takes a parameter, and reports what `main`
/// returns as Rust's entry point does.
#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Some(t) = attr.into_iter().next() {
        return error(t.span(), "`#[tide::main]` takes no arguments");
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
        return error(span, "`#[tide::main]` goes on the binary's `fn main`");
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
fn error(span: Span, message: &str) -> TokenStream {
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
