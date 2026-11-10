//! `Error`'s syntax and generated impls (reference 2.2). Rust owns format checking.
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Expr, Fields, Generics, Ident, LitStr, Member, Token, Type,
};

type Result<T> = syn::Result<T>;

enum Message {
    Transparent(Span),
    Format(LitStr, TokenStream),
}

struct Argument {
    name: Option<Ident>,
    value: Expr,
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> Result<Self> {
        let name =
            if input.peek(Ident::peek_any) && input.peek2(Token![=]) && !input.peek2(Token![==]) {
                let name = input.call(Ident::parse_any)?;
                input.parse::<Token![=]>()?;
                Some(name)
            } else {
                None
            };
        Ok(Self {
            name,
            value: input.parse()?,
        })
    }
}

fn message(attrs: &[Attribute]) -> Result<Option<Message>> {
    let mut result = None;
    for attr in attrs {
        if attr.path().is_ident("source") || attr.path().is_ident("from") {
            return Err(syn::Error::new_spanned(
                attr,
                "#[source] and #[from] are only allowed on a field",
            ));
        }
        if !attr.path().is_ident("error") {
            continue;
        }
        if result.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "duplicate #[error] attribute",
            ));
        }
        result = Some(attr.parse_args_with(|input: ParseStream| {
            if input.peek(LitStr) {
                let fmt = input.parse()?;
                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }
                let rest: TokenStream = input.parse()?;
                Ok(Message::Format(fmt, rest))
            } else if input.peek(Ident) {
                let word: Ident = input.parse()?;
                if word != "transparent" {
                    return Err(syn::Error::new(
                        word.span(),
                        "expected a string literal or `transparent`",
                    ));
                }
                Ok(Message::Transparent(attr.span()))
            } else {
                Err(input.error("expected a string literal or `transparent`"))
            }
        })?);
    }
    Ok(result)
}

fn binding(member: &Member) -> Ident {
    match member {
        Member::Named(n) => n.clone(),
        Member::Unnamed(n) => format_ident!("_{}", n.index, span = n.span),
    }
}

fn members(fields: &Fields) -> Vec<Member> {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            f.ident
                .clone()
                .map(Member::Named)
                .unwrap_or_else(|| Member::Unnamed(i.into()))
        })
        .collect()
}

// `.field` in an argument is shorthand only at the start of an expression.
fn rewrite_fields(input: TokenStream, fields: &Fields) -> TokenStream {
    let tokens: Vec<_> = input.into_iter().collect();
    let mut out = TokenStream::new();
    let mut i = 0;
    let mut starts_expr = true;
    let mut turbofish = 0usize;
    while i < tokens.len() {
        if starts_expr
            && matches!(&tokens[i], TokenTree::Punct(p) if p.as_char() == '.')
            && i + 1 < tokens.len()
        {
            // Rust lexes nested tuple members `.0.0` as a dot and a float token.
            if let TokenTree::Literal(lit) = &tokens[i + 1] {
                let text = lit.to_string();
                if let Some((first, second)) = text.split_once('.')
                    && let (Ok(member), Ok(second)) = (
                        syn::parse_str::<Member>(first),
                        syn::parse_str::<Member>(second),
                    )
                    && members(fields).contains(&member)
                {
                    let name = binding(&member);
                    out.extend(quote_spanned!(lit.span()=> #name.#second));
                    i += 2;
                    starts_expr = false;
                    continue;
                }
            }
            if let Ok(member) = syn::parse2::<Member>(tokens[i + 1].clone().into()) {
                if members(fields).contains(&member) {
                    let name = binding(&member);
                    out.extend(quote_spanned!(tokens[i].span()=> #name));
                    i += 2;
                    starts_expr = false;
                    continue;
                }
            }
        }
        let closes_turbofish = match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == '<' => {
                if turbofish > 0
                    || matches!(tokens[..i].last_chunk(), Some([TokenTree::Punct(a), TokenTree::Punct(b)])
                        if a.as_char() == ':' && a.spacing() == proc_macro2::Spacing::Joint && b.as_char() == ':')
                {
                    turbofish += 1;
                }
                false
            }
            TokenTree::Punct(p)
                if p.as_char() == '>'
                    && turbofish > 0
                    && !matches!(tokens[..i].last(), Some(TokenTree::Punct(p))
                    if p.as_char() == '-' && p.spacing() == proc_macro2::Spacing::Joint) =>
            {
                turbofish -= 1;
                true
            }
            _ => false,
        };
        starts_expr = match &tokens[i] {
            TokenTree::Ident(id) => matches!(
                id.to_string().as_str(),
                "break" | "continue" | "if" | "in" | "match" | "mut" | "return" | "while"
            ),
            TokenTree::Punct(p) => {
                matches!(
                    p.as_char(),
                    '+' | '&' | '!' | '^' | ',' | '/' | '=' | '<' | '|' | '%' | ';' | '*' | '-'
                ) || p.as_char() == '>' && !closes_turbofish
            }
            _ => false,
        };
        let token = match &tokens[i] {
            TokenTree::Group(g) if g.delimiter() != proc_macro2::Delimiter::None => {
                let mut new =
                    proc_macro2::Group::new(g.delimiter(), rewrite_fields(g.stream(), fields));
                new.set_span(g.span());
                TokenTree::Group(new)
            }
            t => t.clone(),
        };
        out.extend([token]);
        i += 1;
    }
    out
}

fn pattern(path: &TokenStream, fields: &Fields) -> TokenStream {
    let members = members(fields);
    let names: Vec<_> = members.iter().map(binding).collect();
    match fields {
        Fields::Named(_) => quote!(#path { #(#names),* }),
        Fields::Unnamed(_) => quote!(#path (#(#names),*)),
        Fields::Unit => quote!(#path),
    }
}

fn uses_generic(ty: &Type, generics: &Generics) -> bool {
    struct Visitor<'a> {
        generics: &'a Generics,
        found: bool,
    }
    impl<'ast> syn::visit::Visit<'ast> for Visitor<'_> {
        fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
            self.found |= path
                .path
                .segments
                .first()
                .is_some_and(|s| self.generics.type_params().any(|p| p.ident == s.ident));
            syn::visit::visit_type_path(self, path);
        }
    }
    let mut visitor = Visitor {
        generics,
        found: false,
    };
    syn::visit::Visit::visit_type(&mut visitor, ty);
    visitor.found
}

fn bound(generics: &mut Generics, original: &Generics, ty: &Type, constraint: TokenStream) {
    if uses_generic(ty, original) {
        generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote!(#ty: #constraint));
    }
}

fn option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(p) = ty {
        let last = p.path.segments.last()?;
        if last.ident == "Option" {
            if let syn::PathArguments::AngleBracketed(args) = &last.arguments {
                if args.args.len() == 1
                    && let Some(syn::GenericArgument::Type(ty)) = args.args.first()
                {
                    return Some(ty);
                }
            }
        }
    }
    None
}

/// Select fields once: explicit source/from wins over the conventional field name.
fn source_field(fields: &Fields) -> Result<(Option<usize>, Option<usize>)> {
    let mut source = None;
    let mut from = None;
    for (i, field) in fields.iter().enumerate() {
        let mut marked = false;
        let mut has_from = false;
        for attr in &field.attrs {
            if attr.path().is_ident("error") {
                return Err(syn::Error::new_spanned(
                    attr,
                    "#[error] is not allowed on a field",
                ));
            }
            if attr.path().is_ident("source") || attr.path().is_ident("from") {
                attr.meta.require_path_only()?;
                let seen = if attr.path().is_ident("from") {
                    &mut has_from
                } else {
                    &mut marked
                };
                if *seen {
                    return Err(syn::Error::new_spanned(attr, "duplicate field attribute"));
                }
                *seen = true;
            }
        }
        if marked || has_from {
            if source.replace(i).is_some() {
                return Err(syn::Error::new_spanned(field, "more than one source field"));
            }
        }
        if has_from {
            from = Some(i);
        }
    }
    if source.is_none() {
        source = fields
            .iter()
            .position(|f| f.ident.as_ref().is_some_and(|id| id == "source"));
    }
    if from.is_some() && fields.len() != 1 {
        return Err(syn::Error::new_spanned(
            fields,
            "#[from] requires no fields other than the source",
        ));
    }
    Ok((source, from))
}

fn display(
    fmt: &LitStr,
    args: &TokenStream,
    fields: &Fields,
    generics: &mut Generics,
    original: &Generics,
) -> Result<TokenStream> {
    let args: Vec<Argument> = syn::parse::Parser::parse2(
        |input: ParseStream| {
            Ok(input
                .parse_terminated(Argument::parse, Token![,])?
                .into_iter()
                .collect())
        },
        rewrite_fields(args.clone(), fields),
    )?;
    let formatter = Ident::new("__tide_error_formatter", Span::mixed_site());
    let text = fmt.value();
    let mut out = String::new();
    let mut rest = text.as_str();
    let members = members(fields);
    let mut added = Vec::new();
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start + 1]);
        rest = &rest[start + 1..];
        if let Some(tail) = rest.strip_prefix('{') {
            out.push('{');
            rest = tail;
            continue;
        }
        let Some(end) = rest.find('}') else {
            break;
        };
        let placeholder = &rest[..end];
        let (key, spec) = placeholder.split_once(':').unwrap_or((placeholder, ""));
        let explicit = args
            .iter()
            .any(|a| a.name.as_ref().is_some_and(|n| n.unraw() == key));
        let positional = if !key.is_empty() && key.bytes().all(|b| b.is_ascii_digit()) {
            key.parse::<usize>().ok()
        } else {
            None
        };
        if positional.is_some()
            && matches!(fields, Fields::Unnamed(_))
            && args.iter().any(|a| a.name.is_none())
        {
            return Err(syn::Error::new(
                fmt.span(),
                "ambiguous positional argument; use a named argument",
            ));
        }
        let index = if explicit {
            None
        } else {
            members.iter().position(|m| match m {
                Member::Named(n) => n.unraw() == key,
                Member::Unnamed(n) => Some(n.index as usize) == positional,
            })
        };
        if let Some(i) = index {
            let mut name = format_ident!("__tide_error_field_{}", added.len(), span = fmt.span());
            while args.iter().any(|a| a.name.as_ref() == Some(&name)) {
                name = format_ident!("_{}", name, span = fmt.span());
            }
            out.push_str(&name.to_string());
            if placeholder.contains(':') {
                out.push(':');
                out.push_str(spec);
            }
            let field = &fields.iter().nth(i).unwrap().ty;
            let binding = binding(&members[i]);
            let trait_name = match spec.chars().last() {
                Some('?') => "Debug",
                Some('x') => "LowerHex",
                Some('X') => "UpperHex",
                Some('o') => "Octal",
                Some('b') => "Binary",
                Some('p') => "Pointer",
                Some('e') => "LowerExp",
                Some('E') => "UpperExp",
                _ => "Display",
            };
            let trait_ident = Ident::new(trait_name, fmt.span());
            bound(generics, original, field, quote!(::core::fmt::#trait_ident));
            let value = if !placeholder.contains(':') {
                quote!(#binding.as_display())
            } else if trait_name == "Pointer" {
                quote!(::std::__error::Pointer(#binding))
            } else {
                quote!(#binding)
            };
            added.push(quote!(#name = #value));
        } else {
            out.push_str(placeholder);
        }
        out.push('}');
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    let fmt = LitStr::new(&out, fmt.span());
    let args = args.iter().map(|a| {
        let value = &a.value;
        if let Some(name) = &a.name {
            quote!(#name = #value)
        } else {
            quote!(#value)
        }
    });
    Ok(quote! {
        use ::std::__error::AsDisplay as _;
        ::core::write!(#formatter, #fmt #(, #args)* #(, #added)*)
    })
}

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let original = &input.generics;
    let mut display_generics = original.clone();
    let mut error_generics = original.clone();
    let default = message(&input.attrs)?;
    let bodies: Vec<_> = match &input.data {
        Data::Struct(s) => vec![(quote!(Self), &s.fields, name, None)],
        Data::Enum(e) => e
            .variants
            .iter()
            .map(|v| {
                let id = &v.ident;
                Ok((quote!(Self::#id), &v.fields, id, message(&v.attrs)?))
            })
            .collect::<Result<_>>()?,
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "Error cannot be derived for a union",
            ));
        }
    };
    let formatter = Ident::new("__tide_error_formatter", Span::mixed_site());
    let mut displays = Vec::new();
    let mut sources = Vec::new();
    let mut conversions = TokenStream::new();
    let mut has_display = true;
    for (path, fields, _, msg) in &bodies {
        let msg = msg.as_ref().or(default.as_ref());
        let (source, from) = source_field(fields)?;
        let pat = pattern(path, fields);
        let all_members = members(fields);
        let source_body = if let Some(Message::Transparent(span)) = msg {
            if fields.len() != 1 {
                return Err(syn::Error::new(
                    *span,
                    "#[error(transparent)] requires exactly one field",
                ));
            }
            if let Some(attr) = fields
                .iter()
                .next()
                .unwrap()
                .attrs
                .iter()
                .find(|a| a.path().is_ident("source"))
            {
                return Err(syn::Error::new_spanned(
                    attr,
                    "transparent error cannot contain #[source]",
                ));
            }
            let binding = binding(&all_members[0]);
            let ty = &fields.iter().next().unwrap().ty;
            bound(
                &mut display_generics,
                original,
                ty,
                quote!(::core::fmt::Display),
            );
            bound(
                &mut error_generics,
                original,
                ty,
                quote!(::core::error::Error),
            );
            displays.push(quote!(#pat => ::core::fmt::Display::fmt(#binding, #formatter)));
            quote!(::core::error::Error::source(#binding.as_dyn_error()))
        } else {
            if let Some(Message::Format(fmt, args)) = msg {
                let body = display(fmt, args, fields, &mut display_generics, original)?;
                displays.push(quote!(#pat => { #body }));
            } else {
                has_display = false;
            }
            if let Some(i) = source {
                let field = fields.iter().nth(i).unwrap();
                let name = binding(&all_members[i]);
                let ty = option_inner(&field.ty).unwrap_or(&field.ty);
                bound(
                    &mut error_generics,
                    original,
                    ty,
                    quote!(::core::error::Error + 'static),
                );
                if option_inner(&field.ty).is_some() {
                    quote!(#name.as_ref().map(|__tide_error_source| __tide_error_source.as_dyn_error()))
                } else {
                    quote!(::core::option::Option::Some(#name.as_dyn_error()))
                }
            } else {
                quote!(::core::option::Option::None)
            }
        };
        sources.push(quote!(#pat => { #source_body }));
        if let Some(i) = from {
            let field = fields.iter().nth(i).unwrap();
            let member = &all_members[i];
            let ty = option_inner(&field.ty).unwrap_or(&field.ty);
            let value = if option_inner(&field.ty).is_some() {
                quote!(::core::option::Option::Some(__tide_error_source))
            } else {
                quote!(__tide_error_source)
            };
            let (imp, ty_args, wh) = original.split_for_impl();
            conversions.extend(quote_spanned! { field.span()=>
                #[automatically_derived]
                impl #imp ::core::convert::From<#ty> for #name #ty_args #wh {
                    fn from(__tide_error_source: #ty) -> Self { #path { #member: #value } }
                }
            });
        }
    }
    if !has_display && bodies.iter().any(|(_, _, _, m)| m.is_some()) {
        let (_, _, variant, _) = bodies.iter().find(|(_, _, _, m)| m.is_none()).unwrap();
        return Err(syn::Error::new_spanned(
            variant,
            "missing #[error] attribute on variant",
        ));
    }
    let matched = if bodies.is_empty() {
        quote!(*self)
    } else {
        quote!(self)
    };
    let (imp, ty_args, wh) = display_generics.split_for_impl();
    let display_impl = has_display.then(|| {
        quote! {
            #[automatically_derived]
            #[allow(unused_variables)]
            impl #imp ::core::fmt::Display for #name #ty_args #wh {
                fn fmt(&self, #formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match #matched { #(#displays),* }
                }
            }
        }
    });
    // Error's supertraits must have the bounds inferred independently by Display/Debug.
    let (_, ty_args, _) = original.split_for_impl();
    if !original.params.is_empty() {
        error_generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote!(Self: ::core::fmt::Debug + ::core::fmt::Display));
    }
    let (imp, _, wh) = error_generics.split_for_impl();
    Ok(quote! {
        #display_impl
        #[automatically_derived]
        #[allow(unused_variables)]
        impl #imp ::core::error::Error for #name #ty_args #wh {
            fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
                use ::std::__error::AsDynError as _;
                match #matched { #(#sources),* }
            }
        }
        #conversions
    })
}
