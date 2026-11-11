//! `Encode` and `Decode` (reference 2.2, bead `fx-08m`): `#[codec]`'s syntax and the generated
//! impls, which call Tide's std's hidden `__codec` engine. The JSON mapping and the attribute
//! semantics are serde's; both derives read `#[codec]` through `attrs`, so they agree on names.
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Fields, Generics, Ident, LitStr, Token, Type};

type Result<T> = syn::Result<T>;

const KEYS: &str =
    "`#[codec]` takes `rename = \"..\"`, `rename_all = \"..\"`, `default`, and `skip`";

/// `rename_all`'s values. The table and the two conversions are ported from serde
/// (`serde_derive/src/internals/case.rs`, MIT OR Apache-2.0, https://github.com/serde-rs/serde),
/// since "serde's spellings" is the contract. They assume `snake_case` fields and `PascalCase`
/// variants and do not split words otherwise. Where serde slices the first byte, this takes
/// the first character, which differs only for names serde panics on.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Case {
    Lower,
    Upper,
    Pascal,
    Camel,
    Snake,
    ScreamingSnake,
    Kebab,
    ScreamingKebab,
}

const CASES: &[(&str, Case)] = &[
    ("lowercase", Case::Lower),
    ("UPPERCASE", Case::Upper),
    ("PascalCase", Case::Pascal),
    ("camelCase", Case::Camel),
    ("snake_case", Case::Snake),
    ("SCREAMING_SNAKE_CASE", Case::ScreamingSnake),
    ("kebab-case", Case::Kebab),
    ("SCREAMING-KEBAB-CASE", Case::ScreamingKebab),
];

fn lower_first(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

impl Case {
    fn variant(self, variant: &str) -> String {
        match self {
            Case::Pascal => variant.to_owned(),
            Case::Lower => variant.to_ascii_lowercase(),
            Case::Upper => variant.to_ascii_uppercase(),
            Case::Camel => lower_first(variant),
            Case::Snake => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            }
            Case::ScreamingSnake => Case::Snake.variant(variant).to_ascii_uppercase(),
            Case::Kebab => Case::Snake.variant(variant).replace('_', "-"),
            Case::ScreamingKebab => Case::ScreamingSnake.variant(variant).replace('_', "-"),
        }
    }

    fn field(self, field: &str) -> String {
        match self {
            Case::Lower | Case::Snake => field.to_owned(),
            Case::Upper | Case::ScreamingSnake => field.to_ascii_uppercase(),
            Case::Pascal => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            Case::Camel => lower_first(&Case::Pascal.field(field)),
            Case::Kebab => field.replace('_', "-"),
            Case::ScreamingKebab => Case::ScreamingSnake.field(field).replace('_', "-"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Place {
    Struct,
    UnitStruct,
    NewtypeStruct,
    Enum,
    Variant,
    Field,
    TupleField,
}

#[derive(Default)]
struct Attrs {
    rename: Option<String>,
    rename_all: Option<Case>,
    default: bool,
    skip: bool,
}

/// The `#[codec(..)]` keys on one item. Every key outside the reference's four, and every key
/// where it has no meaning, is an error at the key.
fn attrs(attrs: &[Attribute], place: Place) -> Result<Attrs> {
    let mut out = Attrs::default();
    for attr in attrs.iter().filter(|a| a.path().is_ident("codec")) {
        if place == Place::TupleField {
            return Err(syn::Error::new_spanned(
                attr,
                "`#[codec]` is not allowed on a tuple field; it goes on a named field",
            ));
        }
        attr.parse_nested_meta(|meta| {
            let key = meta.path.get_ident().map(Ident::to_string);
            let (allowed, home, seen): (&[Place], _, _) = match key.as_deref() {
                Some("rename") => (
                    &[Place::Field, Place::Variant],
                    "a field or a variant",
                    out.rename.is_some(),
                ),
                Some("rename_all") => (
                    &[
                        Place::Struct,
                        Place::UnitStruct,
                        Place::NewtypeStruct,
                        Place::Enum,
                    ],
                    "the struct or enum",
                    out.rename_all.is_some(),
                ),
                Some("default") => (
                    &[Place::Field, Place::Struct],
                    "a field or a struct with named fields or an array representation",
                    out.default,
                ),
                Some("skip") => (&[Place::Field], "a field", out.skip),
                Some(key @ ("flatten" | "deny_unknown_fields" | "try_from")) => {
                    return Err(meta.error(format!("`{key}` is not in v0; {KEYS}")));
                }
                _ => return Err(meta.error(format!("unknown `#[codec]` key; {KEYS}"))),
            };
            let key = key.unwrap();
            if !allowed.contains(&place) {
                let why = if key == "skip" && place == Place::Variant {
                    ", since `encode` cannot fail"
                } else {
                    ""
                };
                return Err(meta.error(format!(
                    "`{key}` is not allowed here{why}; it goes on {home}"
                )));
            }
            if seen {
                return Err(meta.error(format!("duplicate `{key}`")));
            }
            match key.as_str() {
                "rename" => out.rename = Some(meta.value()?.parse::<LitStr>()?.value()),
                "rename_all" => {
                    let value: LitStr = meta.value()?.parse()?;
                    let case = CASES.iter().find(|(name, _)| *name == value.value());
                    let Some((_, case)) = case else {
                        let names: Vec<_> = CASES.iter().map(|(n, _)| format!("\"{n}\"")).collect();
                        return Err(syn::Error::new(
                            value.span(),
                            format!(
                                "unknown `rename_all` value; expected one of {}",
                                names.join(", ")
                            ),
                        ));
                    };
                    out.rename_all = Some(*case);
                }
                _ => {
                    if meta.input.peek(Token![=]) || meta.input.peek(syn::token::Paren) {
                        return Err(meta.error(format!("`{key}` takes no value; {KEYS}")));
                    }
                    if key == "default" {
                        out.default = true;
                    } else {
                        out.skip = true;
                    }
                }
            }
            Ok(())
        })?;
    }
    Ok(out)
}

struct Field<'a> {
    /// The field as the type names it: an identifier or an index.
    member: syn::Member,
    ty: &'a Type,
    /// The JSON key of a named field.
    name: String,
    default: bool,
    skip: bool,
}

/// The fields of a struct or variant with their JSON names, which must be distinct.
fn fields(fields: &Fields, rename_all: Option<Case>) -> Result<Vec<Field<'_>>> {
    let mut out: Vec<Field<'_>> = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        let Some(ident) = &field.ident else {
            attrs(&field.attrs, Place::TupleField)?;
            out.push(Field {
                member: syn::Member::Unnamed(i.into()),
                ty: &field.ty,
                name: String::new(),
                default: false,
                skip: false,
            });
            continue;
        };
        let attrs = attrs(&field.attrs, Place::Field)?;
        let plain = ident.unraw().to_string();
        let name = attrs
            .rename
            .unwrap_or_else(|| rename_all.map_or(plain.clone(), |case| case.field(&plain)));
        if !attrs.skip && out.iter().any(|f| !f.skip && f.name == name) {
            return Err(syn::Error::new_spanned(
                ident,
                format!("two fields have the JSON name `{name}`"),
            ));
        }
        out.push(Field {
            member: syn::Member::Named(ident.clone()),
            ty: &field.ty,
            name,
            default: attrs.default,
            skip: attrs.skip,
        });
    }
    Ok(out)
}

struct Variant<'a> {
    ident: &'a Ident,
    name: String,
    shape: &'a Fields,
    fields: Vec<Field<'a>>,
}

struct Input<'a> {
    /// A struct's own fields are a lone variant.
    variants: Vec<Variant<'a>>,
    is_enum: bool,
    /// `#[codec(default)]` on a struct: missing fields come from `Self::default()`.
    default: bool,
}

fn input(input: &DeriveInput) -> Result<Input<'_>> {
    match &input.data {
        Data::Struct(data) => {
            let place = match &data.fields {
                Fields::Unit => Place::UnitStruct,
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Place::NewtypeStruct,
                _ => Place::Struct,
            };
            let container = attrs(&input.attrs, place)?;
            Ok(Input {
                variants: vec![Variant {
                    ident: &input.ident,
                    name: String::new(),
                    shape: &data.fields,
                    fields: fields(&data.fields, container.rename_all)?,
                }],
                is_enum: false,
                default: container.default,
            })
        }
        Data::Enum(data) => {
            let container = attrs(&input.attrs, Place::Enum)?;
            let mut variants: Vec<Variant<'_>> = Vec::new();
            for variant in &data.variants {
                let attrs = attrs(&variant.attrs, Place::Variant)?;
                let plain = variant.ident.unraw().to_string();
                let name = attrs.rename.unwrap_or_else(|| {
                    container
                        .rename_all
                        .map_or(plain.clone(), |case| case.variant(&plain))
                });
                if variants.iter().any(|v| v.name == name) {
                    return Err(syn::Error::new_spanned(
                        &variant.ident,
                        format!("two variants have the JSON name `{name}`"),
                    ));
                }
                variants.push(Variant {
                    ident: &variant.ident,
                    name,
                    shape: &variant.fields,
                    fields: fields(&variant.fields, None)?,
                });
            }
            Ok(Input {
                variants,
                is_enum: true,
                default: false,
            })
        }
        Data::Union(_) => Err(syn::Error::new_spanned(
            &input.ident,
            "Encode and Decode cannot be derived for a union",
        )),
    }
}

/// Bound a projection as a whole, without imposing codec bounds on its provider.
fn codec_types(ty: &Type, generics: &Generics) -> Vec<Type> {
    struct Visitor<'a> {
        generics: &'a Generics,
        types: Vec<Type>,
    }
    impl<'ast> syn::visit::Visit<'ast> for Visitor<'_> {
        fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
            if path.qself.is_some()
                || path.path.segments.first().is_some_and(|first| {
                    self.generics.type_params().any(|p| p.ident == first.ident)
                })
            {
                self.types.push(Type::Path(path.clone()));
            } else {
                syn::visit::visit_type_path(self, path);
            }
        }
    }
    let mut visitor = Visitor {
        generics,
        types: Vec::new(),
    };
    syn::visit::Visit::visit_type(&mut visitor, ty);
    visitor.types
}

/// Bound ordinary parameters and associated projections, not whole recursive field types.
fn generics(input: &DeriveInput, parsed: &Input<'_>, bound: TokenStream, decode: bool) -> Generics {
    let original = &input.generics;
    let mut out = original.clone();
    let all = || parsed.variants.iter().flat_map(|v| &v.fields);
    let mut seen = std::collections::BTreeSet::new();
    for ty in all()
        .filter(|f| !f.skip)
        .flat_map(|f| codec_types(f.ty, original))
    {
        if seen.insert(quote!(#ty).to_string()) {
            out.make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ty: #bound));
        }
    }
    if decode {
        if parsed.default && original.type_params().next().is_some() {
            out.make_where_clause()
                .predicates
                .push(syn::parse_quote!(Self: ::core::default::Default));
        }
        // Elsewhere a missing `Default` is an error at the field.
        for field in all().filter(|f| f.default || (f.skip && !parsed.default)) {
            if !codec_types(field.ty, original).is_empty() {
                let ty = field.ty;
                out.make_where_clause()
                    .predicates
                    .push(syn::parse_quote!(#ty: ::core::default::Default));
            }
        }
    }
    out
}

/// The local for field `i`. It carries the span of the field's type, so that rustc reports an
/// unmet bound at the field and not at the derive.
fn local(field: &Field<'_>, i: usize) -> Ident {
    format_ident!("__tide_codec_field{i}", span = field.ty.span())
}

fn path(input: &Input<'_>, variant: &Variant<'_>) -> TokenStream {
    if input.is_enum {
        let ident = variant.ident;
        quote!(Self::#ident)
    } else {
        quote!(Self)
    }
}

pub fn encode(derive: DeriveInput) -> Result<TokenStream> {
    let parsed = input(&derive)?;
    let out = Ident::new("__tide_codec_out", Span::mixed_site());
    let encoder = quote!(::std::__codec::Encoder);
    let mut arms = Vec::new();
    for variant in &parsed.variants {
        // A struct's fields are read through `self`, a variant's through bindings.
        let values: Vec<TokenStream> = variant
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                if parsed.is_enum {
                    let binding = local(f, i);
                    quote!(#binding)
                } else {
                    let member = &f.member;
                    quote_spanned!(f.ty.span()=> &self.#member)
                }
            })
            .collect();
        let coded = || variant.fields.iter().zip(&values).filter(|(f, _)| !f.skip);
        let body = match variant.shape {
            Fields::Named(_) => {
                let fields = coded().map(|(f, value)| {
                    let name = &f.name;
                    quote_spanned!(f.ty.span()=> ::std::__codec::Encoder::field(#out, #name, #value);)
                });
                quote! {
                    #encoder::object_begin(#out);
                    #(#fields)*
                    #encoder::object_end(#out);
                }
            }
            Fields::Unnamed(_) if values.len() == 1 => {
                let (f, value) = coded().next().unwrap();
                quote_spanned!(f.ty.span()=> ::std::__codec::Encode::__encode(#value, #out);)
            }
            Fields::Unnamed(_) => {
                let elements = coded().map(|(f, value)| {
                    quote_spanned!(f.ty.span()=> ::std::__codec::Encoder::element(#out, #value);)
                });
                quote! {
                    #encoder::array_begin(#out);
                    #(#elements)*
                    #encoder::array_end(#out);
                }
            }
            Fields::Unit if parsed.is_enum => {
                let name = &variant.name;
                quote!(#encoder::string(#out, #name);)
            }
            Fields::Unit => quote!(#encoder::null(#out);),
        };
        if !parsed.is_enum {
            arms.push(body);
            continue;
        }
        let path = path(&parsed, variant);
        let bindings = coded().map(|(f, value)| {
            let member = &f.member;
            quote!(#member: #value)
        });
        let name = &variant.name;
        let body = if matches!(variant.shape, Fields::Unit) {
            body
        } else {
            quote! {
                #encoder::variant_begin(#out, #name);
                #body
                #encoder::variant_end(#out);
            }
        };
        arms.push(quote!(#path { #(#bindings,)* .. } => { #body }));
    }
    let body = if parsed.is_enum {
        // `match *self {}` for an enum with no variants.
        let matched = if arms.is_empty() {
            quote!(*self)
        } else {
            quote!(self)
        };
        quote!(match #matched { #(#arms)* })
    } else {
        quote!(#(#arms)*)
    };
    let name = &derive.ident;
    let generics = generics(&derive, &parsed, quote!(::std::__codec::Encode), false);
    let (imp, _, wh) = generics.split_for_impl();
    let (_, ty_args, _) = derive.generics.split_for_impl();
    Ok(quote! {
        #[automatically_derived]
        #[allow(unused_variables)]
        impl #imp ::std::__codec::Encode for #name #ty_args #wh {
            fn __encode(&self, #out: &mut #encoder) {
                #body
            }
        }
    })
}

/// A block that decodes one struct's or variant's fields and evaluates to the value.
fn decode_fields(input: &Input<'_>, variant: &Variant<'_>, decoder: &Ident) -> TokenStream {
    let path = path(input, variant);
    let engine = quote!(::std::__codec::Decoder);
    let fields = &variant.fields;
    let locals: Vec<Ident> = fields
        .iter()
        .enumerate()
        .map(|(i, f)| local(f, i))
        .collect();
    let decoded =
        |f: &Field<'_>| quote_spanned!(f.ty.span()=> ::std::__codec::Decoder::decode(#decoder)?);
    match variant.shape {
        Fields::Unit => quote!({
            #engine::null(#decoder)?;
            #path
        }),
        Fields::Unnamed(_) if fields.len() == 1 => {
            let value = decoded(&fields[0]);
            quote!(#path(#value))
        }
        Fields::Unnamed(_) if input.default => {
            let len = fields.len();
            let fallback = Ident::new("__tide_codec_default", Span::mixed_site());
            let ended = Ident::new("__tide_codec_ended", Span::mixed_site());
            let values = fields.iter().map(|f| {
                let value = decoded(f);
                let member = &f.member;
                quote! {
                    if !#ended && #engine::next_element(#decoder)? {
                        #value
                    } else {
                        #ended = true;
                        #fallback.#member
                    }
                }
            });
            quote!({
                #engine::array_begin(#decoder)?;
                let #fallback: Self = ::core::default::Default::default();
                let mut #ended = false;
                #(let #locals = #values;)*
                if !#ended {
                    #engine::tuple_end(#decoder, #len)?;
                }
                #path(#(#locals),*)
            })
        }
        Fields::Unnamed(_) => {
            let len = fields.len();
            let values = fields.iter().map(decoded);
            quote!({
                #engine::array_begin(#decoder)?;
                #(let #locals = {
                    #engine::tuple_next(#decoder, #len)?;
                    #values
                };)*
                #engine::tuple_end(#decoder, #len)?;
                #path(#(#locals),*)
            })
        }
        Fields::Named(_) => {
            let fallback = Ident::new("__tide_codec_default", Span::mixed_site());
            let coded: Vec<_> = fields
                .iter()
                .zip(&locals)
                .filter(|(f, _)| !f.skip)
                .collect();
            let slots = coded.iter().map(|(f, slot)| {
                let ty = f.ty;
                quote!(let mut #slot: ::core::option::Option<#ty> = ::core::option::Option::None;)
            });
            let names = coded.iter().map(|(f, _)| &f.name);
            let arms = coded.iter().enumerate().map(|(i, (f, slot))| {
                let name = &f.name;
                quote_spanned!(f.ty.span()=> #i => ::std::__codec::Decoder::field(#decoder, &mut #slot, #name)?,)
            });
            let values = fields.iter().zip(&locals).map(|(f, slot)| {
                let member = &f.member;
                let name = &f.name;
                let absent = if input.default && !f.default {
                    quote!(#fallback.#member)
                } else {
                    quote_spanned!(f.ty.span()=> ::core::default::Default::default())
                };
                let value = if f.skip {
                    absent
                } else if f.default || input.default {
                    quote! {
                        match #slot {
                            ::core::option::Option::Some(value) => value,
                            ::core::option::Option::None => #absent,
                        }
                    }
                } else {
                    quote_spanned!(f.ty.span()=> ::std::__codec::Decoder::required(#decoder, #slot, #name)?)
                };
                quote!(#member: #value)
            });
            let fallback = input
                .default
                .then(|| quote!(let #fallback: Self = ::core::default::Default::default();));
            let index = Ident::new("__tide_codec_index", Span::mixed_site());
            quote!({
                #fallback
                #(#slots)*
                #engine::object_begin(#decoder)?;
                while let ::core::option::Option::Some(#index) =
                    #engine::next_field(#decoder, &[#(#names),*])?
                {
                    match #index {
                        #(#arms)*
                        _ => #engine::skip_value(#decoder)?,
                    }
                }
                #path { #(#values),* }
            })
        }
    }
}

pub fn decode(derive: DeriveInput) -> Result<TokenStream> {
    let parsed = input(&derive)?;
    let decoder = Ident::new("__tide_codec_input", Span::mixed_site());
    let engine = quote!(::std::__codec::Decoder);
    let body = if parsed.is_enum {
        let tags = parsed.variants.iter().map(|v| {
            let name = &v.name;
            let data = !matches!(v.shape, Fields::Unit);
            quote!((#name, #data))
        });
        let arms = parsed.variants.iter().enumerate().map(|(i, v)| {
            if matches!(v.shape, Fields::Unit) {
                let path = path(&parsed, v);
                quote!(#i => ::core::result::Result::Ok(#path),)
            } else {
                let value = decode_fields(&parsed, v, &decoder);
                quote! {
                    #i => {
                        let value = #value;
                        #engine::variant_end(#decoder)?;
                        ::core::result::Result::Ok(value)
                    }
                }
            }
        });
        quote! {
            match #engine::variant_begin(#decoder, &[#(#tags),*])? {
                #(#arms)*
                _ => ::core::result::Result::Err(#engine::invalid_variant(#decoder)),
            }
        }
    } else {
        let value = decode_fields(&parsed, &parsed.variants[0], &decoder);
        quote!(::core::result::Result::Ok(#value))
    };
    let name = &derive.ident;
    let generics = generics(&derive, &parsed, quote!(::std::__codec::Decode), true);
    let (imp, _, wh) = generics.split_for_impl();
    let (_, ty_args, _) = derive.generics.split_for_impl();
    Ok(quote! {
        #[automatically_derived]
        #[allow(unused_variables, unused_mut)]
        impl #imp ::std::__codec::Decode for #name #ty_args #wh {
            fn __decode(
                #decoder: &mut #engine<'_>,
            ) -> ::core::result::Result<Self, ::std::json::Error> {
                #body
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::Case::*;

    // serde's own vectors (`case.rs`, `rename_variants` and `rename_fields`).
    #[test]
    fn cases_convert_as_serde() {
        for (original, lower, upper, camel, snake, screaming, kebab, screaming_kebab) in [
            (
                "Outcome", "outcome", "OUTCOME", "outcome", "outcome", "OUTCOME", "outcome",
                "OUTCOME",
            ),
            (
                "VeryTasty",
                "verytasty",
                "VERYTASTY",
                "veryTasty",
                "very_tasty",
                "VERY_TASTY",
                "very-tasty",
                "VERY-TASTY",
            ),
            ("A", "a", "A", "a", "a", "A", "a", "A"),
            ("Z42", "z42", "Z42", "z42", "z42", "Z42", "z42", "Z42"),
        ] {
            assert_eq!(Lower.variant(original), lower);
            assert_eq!(Upper.variant(original), upper);
            assert_eq!(Pascal.variant(original), original);
            assert_eq!(Camel.variant(original), camel);
            assert_eq!(Snake.variant(original), snake);
            assert_eq!(ScreamingSnake.variant(original), screaming);
            assert_eq!(Kebab.variant(original), kebab);
            assert_eq!(ScreamingKebab.variant(original), screaming_kebab);
        }
        for (original, upper, pascal, camel, screaming, kebab, screaming_kebab) in [
            (
                "outcome", "OUTCOME", "Outcome", "outcome", "OUTCOME", "outcome", "OUTCOME",
            ),
            (
                "very_tasty",
                "VERY_TASTY",
                "VeryTasty",
                "veryTasty",
                "VERY_TASTY",
                "very-tasty",
                "VERY-TASTY",
            ),
            ("a", "A", "A", "a", "A", "a", "A"),
            ("z42", "Z42", "Z42", "z42", "Z42", "z42", "Z42"),
        ] {
            assert_eq!(Lower.field(original), original);
            assert_eq!(Upper.field(original), upper);
            assert_eq!(Pascal.field(original), pascal);
            assert_eq!(Camel.field(original), camel);
            assert_eq!(Snake.field(original), original);
            assert_eq!(ScreamingSnake.field(original), screaming);
            assert_eq!(Kebab.field(original), kebab);
            assert_eq!(ScreamingKebab.field(original), screaming_kebab);
        }
        // Names serde's first-byte slice panics on.
        assert_eq!(Camel.variant("Élan"), "Élan");
        assert_eq!(Camel.field("_"), "");
    }
}
