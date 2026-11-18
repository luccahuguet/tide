//! The engine behind the `Encode` and `Decode` derives and `std::json` (reference 2.2, bead
//! `fx-08m`): the two traits, a JSON writer and reader, and the impls for Rust's and Tide's std
//! types. The conformance corpus's fixture std shares this file through `root.rs`.
//!
//! The traits are public, and in the prelude beside their derives, for bounds such as
//! `T: Decode`. Their methods, `Encoder`, and `Decoder` are hidden and JSON-only: the derives
//! call them, and D6's later format plug-in API may replace them.
//!
//! Types with impls: `bool`; every integer type; `f32` and `f64`; `char` (a one-character
//! string); `String`, and `str` and `&T` for `Encode` only; `()` (`null`); `Option<T>`;
//! `Box<T>`; `Vec<T>`, `VecDeque<T>`, and `[T]` (`Encode` only) as arrays; tuples of one to
//! four elements as arrays; and `HashMap<String, V, S>` and `BTreeMap<String, V>` as objects.
//! Map keys are `String` only, sets have no impls, and no capability or handle type has one
//! (03 ROOT-4). `std::json` documents the encoding.

use crate::collections::{BTreeMap, HashMap, VecDeque};
use crate::json::Error;
use std::fmt::Display;
use std::hash::BuildHasher;
use std::io::Write as _;
use std::str::FromStr;

/// The nesting of arrays and objects a decoder accepts, as serde_json's default.
const DEPTH_LIMIT: usize = 128;

/// Type recursion also includes wrappers that consume no JSON container.
const DECODE_LIMIT: usize = 256;

/// A type that `std::json::encode` writes. Derive it: `#[derive(Encode)]`.
pub trait Encode {
    #[doc(hidden)]
    fn __encode(&self, out: &mut Encoder);
}

/// A type that `std::json::decode` builds, as an owned value. Derive it: `#[derive(Decode)]`.
pub trait Decode: Sized {
    #[doc(hidden)]
    fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error>;

    /// The value of a struct field whose key is absent: an error, except for `Option`.
    #[doc(hidden)]
    fn __missing(input: &Decoder<'_>, field: &str) -> Result<Self, Error> {
        Err(input.error(format!("missing field `{field}`")))
    }
}

/// Writes compact JSON.
pub struct Encoder {
    out: Vec<u8>,
}

impl Encoder {
    pub(crate) fn new() -> Self {
        Self { out: Vec::new() }
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.out
    }

    /// A comma, unless the next item is its container's first. No value ends in `{` or `[`.
    fn separate(&mut self) {
        if !matches!(self.out.last(), Some(b'{' | b'[')) {
            self.out.push(b',');
        }
    }

    fn display(&mut self, value: impl Display) {
        // Writing to a `Vec` does not fail.
        let _ = write!(self.out, "{value}");
    }

    pub fn null(&mut self) {
        self.out.extend_from_slice(b"null");
    }

    /// RFC 8259 §7's required escapes only: `"`, `\`, and U+0000 to U+001F.
    pub fn string(&mut self, s: &str) {
        self.out.push(b'"');
        let mut rest = s.as_bytes();
        while let Some(i) = rest
            .iter()
            .position(|&b| b == b'"' || b == b'\\' || b < 0x20)
        {
            self.out.extend_from_slice(&rest[..i]);
            match rest[i] {
                b'"' => self.out.extend_from_slice(b"\\\""),
                b'\\' => self.out.extend_from_slice(b"\\\\"),
                0x08 => self.out.extend_from_slice(b"\\b"),
                0x09 => self.out.extend_from_slice(b"\\t"),
                0x0a => self.out.extend_from_slice(b"\\n"),
                0x0c => self.out.extend_from_slice(b"\\f"),
                0x0d => self.out.extend_from_slice(b"\\r"),
                b => self.display(format_args!("\\u{b:04x}")),
            }
            rest = &rest[i + 1..];
        }
        self.out.extend_from_slice(rest);
        self.out.push(b'"');
    }

    pub fn object_begin(&mut self) {
        self.out.push(b'{');
    }

    pub fn field<T: Encode + ?Sized>(&mut self, name: &str, value: &T) {
        self.separate();
        self.string(name);
        self.out.push(b':');
        value.__encode(self);
    }

    pub fn object_end(&mut self) {
        self.out.push(b'}');
    }

    pub fn array_begin(&mut self) {
        self.out.push(b'[');
    }

    pub fn element<T: Encode + ?Sized>(&mut self, value: &T) {
        self.separate();
        value.__encode(self);
    }

    pub fn array_end(&mut self) {
        self.out.push(b']');
    }

    /// `{"name":`, before a data variant's value.
    pub fn variant_begin(&mut self, name: &str) {
        self.out.push(b'{');
        self.string(name);
        self.out.push(b':');
    }

    pub fn variant_end(&mut self) {
        self.out.push(b'}');
    }
}

/// Reads JSON (RFC 8259) from bytes.
pub struct Decoder<'a> {
    input: &'a [u8],
    pos: usize,
    depth: usize,
    calls: usize,
    /// Set between a container's opening bracket and the request for its first item.
    fresh: bool,
}

impl<'a> Decoder<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            depth: 0,
            calls: 0,
            fresh: false,
        }
    }

    /// Dispatch every decoded value here, so transparent types cannot bypass the limit.
    pub fn decode<T: Decode>(&mut self) -> Result<T, Error> {
        if self.calls == DECODE_LIMIT {
            return Err(self.error("recursion limit exceeded"));
        }
        self.calls += 1;
        let value = T::__decode(self);
        self.calls -= 1;
        value
    }

    /// Only whitespace may follow the top-level value.
    pub(crate) fn finish(&mut self) -> Result<(), Error> {
        self.ws();
        match self.peek() {
            None => Ok(()),
            Some(_) => Err(self.error("trailing characters")),
        }
    }

    /// An error at the current byte.
    fn error(&self, message: impl Into<String>) -> Error {
        let before = &self.input[..self.pos];
        let line_start = before
            .iter()
            .rposition(|&b| b == b'\n')
            .map_or(0, |i| i + 1);
        Error::new(
            message.into(),
            1 + before.iter().filter(|&&b| b == b'\n').count(),
            1 + self.pos - line_start,
        )
    }

    fn error_at(&mut self, pos: usize, message: impl Into<String>) -> Error {
        self.pos = pos;
        self.error(message)
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn ws(&mut self) {
        while let Some(b' ' | b'\t' | b'\n' | b'\r') = self.peek() {
            self.pos += 1;
        }
    }

    /// The error for a value that is not `expected`, or for no value at all.
    fn invalid_type(&mut self, expected: &str) -> Error {
        self.ws();
        let found = match self.peek() {
            None => return self.error("EOF while parsing a value"),
            Some(b'"') => "string",
            Some(b'{') => "map",
            Some(b'[') => "sequence",
            Some(b't' | b'f') => "boolean",
            Some(b'n') => "null",
            Some(b'-' | b'0'..=b'9') => "number",
            Some(_) => return self.error("expected value"),
        };
        self.error(format!("invalid type: {found}, expected {expected}"))
    }

    fn literal(&mut self, word: &[u8]) -> Result<(), Error> {
        let rest = &self.input[self.pos..];
        if rest.starts_with(word) {
            self.pos += word.len();
            Ok(())
        } else if word.starts_with(rest) {
            Err(self.error_at(self.input.len(), "EOF while parsing a value"))
        } else {
            Err(self.error("expected ident"))
        }
    }

    pub fn null(&mut self) -> Result<(), Error> {
        self.ws();
        match self.peek() {
            Some(b'n') => self.literal(b"null"),
            _ => Err(self.invalid_type("null")),
        }
    }

    fn bool(&mut self) -> Result<bool, Error> {
        self.ws();
        match self.peek() {
            Some(b't') => self.literal(b"true").map(|()| true),
            Some(b'f') => self.literal(b"false").map(|()| false),
            _ => Err(self.invalid_type("a boolean")),
        }
    }

    fn digits(&mut self) -> Result<(), Error> {
        let start = self.pos;
        while let Some(b'0'..=b'9') = self.peek() {
            self.pos += 1;
        }
        match self.peek() {
            _ if self.pos > start => Ok(()),
            None => Err(self.error("EOF while parsing a value")),
            Some(_) => Err(self.error("invalid number")),
        }
    }

    /// A number token of RFC 8259 §6's grammar, which is narrower than Rust's, and whether it
    /// is an integer: it has no fraction and no exponent.
    fn number(&mut self, expected: &str) -> Result<(&'a str, bool), Error> {
        self.ws();
        let start = self.pos;
        if !matches!(self.peek(), Some(b'-' | b'0'..=b'9')) {
            return Err(self.invalid_type(expected));
        }
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        if self.peek() == Some(b'0') {
            self.pos += 1;
            if let Some(b'0'..=b'9') = self.peek() {
                return Err(self.error("invalid number"));
            }
        } else {
            self.digits()?;
        }
        let mut integer = true;
        if self.peek() == Some(b'.') {
            integer = false;
            self.pos += 1;
            self.digits()?;
        }
        if let Some(b'e' | b'E') = self.peek() {
            integer = false;
            self.pos += 1;
            if let Some(b'+' | b'-') = self.peek() {
                self.pos += 1;
            }
            self.digits()?;
        }
        let token = std::str::from_utf8(&self.input[start..self.pos]).expect("ASCII");
        Ok((token, integer))
    }

    /// An integer: a float token is a type error, and a value outside `T` is out of range.
    fn integer<T: FromStr>(&mut self, expected: &str) -> Result<T, Error> {
        self.ws();
        let start = self.pos;
        let (token, integer) = self.number(expected)?;
        if !integer {
            let message = format!("invalid type: floating point `{token}`, expected {expected}");
            return Err(self.error_at(start, message));
        }
        // Unsigned types do not parse a sign.
        let token = if token == "-0" { "0" } else { token };
        token
            .parse()
            .map_err(|_| self.error_at(start, "number out of range"))
    }

    /// A float, rounded once to `T`. Rust parses every token of JSON's grammar.
    fn float<T: FromStr>(&mut self, expected: &str) -> Result<(T, usize), Error> {
        self.ws();
        let start = self.pos;
        let (token, _) = self.number(expected)?;
        match token.parse() {
            Ok(value) => Ok((value, start)),
            Err(_) => Err(self.error_at(start, "invalid number")),
        }
    }

    fn hex4(&mut self) -> Result<u32, Error> {
        let mut value = 0;
        for _ in 0..4 {
            let digit = match self.peek() {
                None => return Err(self.error("EOF while parsing a string")),
                Some(b) => (b as char).to_digit(16),
            };
            let Some(digit) = digit else {
                return Err(self.error("invalid escape"));
            };
            value = value * 16 + digit;
            self.pos += 1;
        }
        Ok(value)
    }

    /// The character of the escape after a `\`. Surrogate pairs combine; a lone one is an error.
    fn escape(&mut self) -> Result<char, Error> {
        let Some(b) = self.peek() else {
            return Err(self.error("EOF while parsing a string"));
        };
        self.pos += 1;
        let scalar = match b {
            b'"' => return Ok('"'),
            b'\\' => return Ok('\\'),
            b'/' => return Ok('/'),
            b'b' => return Ok('\u{8}'),
            b'f' => return Ok('\u{c}'),
            b'n' => return Ok('\n'),
            b'r' => return Ok('\r'),
            b't' => return Ok('\t'),
            b'u' => match self.hex4()? {
                high @ 0xd800..=0xdbff => {
                    if !self.input[self.pos..].starts_with(b"\\u") {
                        return Err(self.error("lone leading surrogate in hex escape"));
                    }
                    self.pos += 2;
                    let low = self.hex4()?;
                    if !(0xdc00..=0xdfff).contains(&low) {
                        return Err(self.error("lone leading surrogate in hex escape"));
                    }
                    0x10000 + ((high - 0xd800) << 10) + (low - 0xdc00)
                }
                0xdc00..=0xdfff => {
                    return Err(self.error("lone trailing surrogate in hex escape"));
                }
                scalar => scalar,
            },
            _ => return Err(self.error_at(self.pos - 1, "invalid escape")),
        };
        Ok(char::from_u32(scalar).expect("a scalar value"))
    }

    /// A string: valid UTF-8 with no raw control character.
    pub fn string(&mut self) -> Result<String, Error> {
        self.ws();
        if self.peek() != Some(b'"') {
            return Err(self.invalid_type("a string"));
        }
        self.pos += 1;
        let mut out = String::new();
        loop {
            let start = self.pos;
            while self
                .peek()
                .is_some_and(|b| b != b'"' && b != b'\\' && b >= 0x20)
            {
                self.pos += 1;
            }
            match std::str::from_utf8(&self.input[start..self.pos]) {
                Ok(run) => out.push_str(run),
                Err(e) => {
                    return Err(
                        self.error_at(start + e.valid_up_to(), "invalid unicode code point")
                    );
                }
            }
            match self.peek() {
                None => return Err(self.error("EOF while parsing a string")),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    out.push(self.escape()?);
                }
                Some(_) => {
                    return Err(self.error(
                        "control character (\\u0000-\\u001F) found while parsing a string",
                    ));
                }
            }
        }
    }

    fn open(&mut self, bracket: u8, expected: &str) -> Result<(), Error> {
        self.ws();
        if self.peek() != Some(bracket) {
            return Err(self.invalid_type(expected));
        }
        if self.depth == DEPTH_LIMIT {
            return Err(self.error("recursion limit exceeded"));
        }
        self.pos += 1;
        self.depth += 1;
        self.fresh = true;
        Ok(())
    }

    /// Moves to the next item of the open container; `false` at its closing bracket.
    fn next_item(&mut self, close: u8, eof: &str, expected: &str) -> Result<bool, Error> {
        let first = std::mem::take(&mut self.fresh);
        self.ws();
        match self.peek() {
            None => Err(self.error(eof)),
            Some(b) if b == close => {
                self.pos += 1;
                self.depth -= 1;
                Ok(false)
            }
            Some(_) if first => Ok(true),
            Some(b',') => {
                self.pos += 1;
                self.ws();
                match self.peek() {
                    None => Err(self.error(eof)),
                    Some(b) if b == close => Err(self.error("trailing comma")),
                    Some(_) => Ok(true),
                }
            }
            Some(_) => Err(self.error(expected)),
        }
    }

    pub fn object_begin(&mut self) -> Result<(), Error> {
        self.open(b'{', "a map")
    }

    /// The next key of the open object, with its colon; `None` at the closing brace.
    fn next_key(&mut self) -> Result<Option<String>, Error> {
        const EOF: &str = "EOF while parsing an object";
        if !self.next_item(b'}', EOF, "expected `,` or `}`")? {
            return Ok(None);
        }
        if self.peek() != Some(b'"') {
            return Err(self.error("key must be a string"));
        }
        let key = self.string()?;
        self.ws();
        match self.peek() {
            Some(b':') => {
                self.pos += 1;
                Ok(Some(key))
            }
            None => Err(self.error(EOF)),
            Some(_) => Err(self.error("expected `:`")),
        }
    }

    /// The next key's index in `names`, or `usize::MAX` for a key that is not among them.
    pub fn next_field(&mut self, names: &[&str]) -> Result<Option<usize>, Error> {
        Ok(self.next_key()?.map(|key| {
            names
                .iter()
                .position(|name| *name == key)
                .unwrap_or(usize::MAX)
        }))
    }

    /// Decodes a known field's value into `slot`. A second occurrence of the key is an error.
    pub fn field<T: Decode>(&mut self, slot: &mut Option<T>, name: &str) -> Result<(), Error> {
        if slot.is_some() {
            return Err(self.error(format!("duplicate field `{name}`")));
        }
        *slot = Some(self.decode()?);
        Ok(())
    }

    /// A field's value after the object ends: the decoded one, or what its type gives an
    /// absent key.
    pub fn required<T: Decode>(&self, slot: Option<T>, name: &str) -> Result<T, Error> {
        match slot {
            Some(value) => Ok(value),
            None => T::__missing(self, name),
        }
    }

    /// Checks and discards one value, as for an unknown key. Nesting counts toward the limit.
    pub fn skip_value(&mut self) -> Result<(), Error> {
        self.ws();
        match self.peek() {
            Some(b'"') => self.string().map(drop),
            Some(b'{') => {
                self.object_begin()?;
                while self.next_key()?.is_some() {
                    self.skip_value()?;
                }
                Ok(())
            }
            Some(b'[') => {
                self.array_begin()?;
                while self.next_element()? {
                    self.skip_value()?;
                }
                Ok(())
            }
            Some(b't') => self.literal(b"true"),
            Some(b'f') => self.literal(b"false"),
            Some(b'n') => self.literal(b"null"),
            _ => self.number("a value").map(drop),
        }
    }

    pub fn array_begin(&mut self) -> Result<(), Error> {
        self.open(b'[', "a sequence")
    }

    /// Whether the open array has another element; `false` consumes its closing bracket.
    pub fn next_element(&mut self) -> Result<bool, Error> {
        self.next_item(b']', "EOF while parsing a list", "expected `,` or `]`")
    }

    /// Moves to the next element of an array that must hold `len` elements.
    pub fn tuple_next(&mut self, len: usize) -> Result<(), Error> {
        if self.next_element()? {
            Ok(())
        } else {
            Err(self.tuple_length(len))
        }
    }

    /// Ends an array that must hold `len` elements.
    pub fn tuple_end(&mut self, len: usize) -> Result<(), Error> {
        if self.next_element()? {
            Err(self.tuple_length(len))
        } else {
            Ok(())
        }
    }

    fn tuple_length(&self, len: usize) -> Error {
        self.error(format!(
            "invalid length, expected an array of {len} elements"
        ))
    }

    /// Reads an externally tagged enum's tag and returns the variant's index in `variants`,
    /// each a name and whether it holds data. A string selects a unit variant, and an object's
    /// only key a data variant, whose value follows and which `variant_end` then closes.
    pub fn variant_begin(&mut self, variants: &[(&str, bool)]) -> Result<usize, Error> {
        let expected = || match variants {
            [] => "a variant, and the enum has none".to_string(),
            [(only, _)] => format!("variant `{only}`"),
            _ => {
                let names: Vec<_> = variants.iter().map(|(n, _)| format!("`{n}`")).collect();
                format!("one of {}", names.join(", "))
            }
        };
        self.ws();
        let start = self.pos;
        let (name, data) = match self.peek() {
            Some(b'"') => (self.string()?, false),
            Some(b'{') => {
                self.object_begin()?;
                match self.next_key()? {
                    Some(key) => (key, true),
                    None => {
                        let message = format!("invalid type: empty map, expected {}", expected());
                        return Err(self.error_at(start, message));
                    }
                }
            }
            _ => return Err(self.invalid_type(&expected())),
        };
        let Some(index) = variants.iter().position(|(n, _)| *n == name) else {
            let message = format!("unknown variant `{name}`, expected {}", expected());
            return Err(self.error_at(start, message));
        };
        if variants[index].1 != data {
            let message = if data {
                format!("invalid type: map, expected the string \"{name}\" for a unit variant")
            } else {
                format!("invalid type: string, expected a single-key map for variant `{name}`")
            };
            return Err(self.error_at(start, message));
        }
        Ok(index)
    }

    pub fn variant_end(&mut self) -> Result<(), Error> {
        match self.next_key()? {
            None => Ok(()),
            Some(_) => Err(self.error("expected a single-key map for an enum")),
        }
    }

    /// For the derive's arm that no index reaches.
    pub fn invalid_variant(&self) -> Error {
        self.error("invalid variant")
    }
}

macro_rules! integers {
    ($($t:ident)*) => {$(
        impl Encode for $t {
            fn __encode(&self, out: &mut Encoder) {
                out.display(self);
            }
        }
        impl Decode for $t {
            fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
                input.integer(stringify!($t))
            }
        }
    )*};
}
integers!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

macro_rules! floats {
    ($($t:ident)*) => {$(
        /// Finite values are written with Rust's shortest round-tripping digits; NaN and
        /// the infinities are `null`.
        impl Encode for $t {
            fn __encode(&self, out: &mut Encoder) {
                if self.is_finite() {
                    out.display(format_args!("{self:?}"));
                } else {
                    out.null();
                }
            }
        }
        /// A decoded float is finite.
        impl Decode for $t {
            fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
                let (value, start): ($t, usize) = input.float(stringify!($t))?;
                if value.is_finite() {
                    Ok(value)
                } else {
                    Err(input.error_at(start, "number out of range"))
                }
            }
        }
    )*};
}
floats!(f32 f64);

impl Encode for bool {
    fn __encode(&self, out: &mut Encoder) {
        out.display(self);
    }
}
impl Decode for bool {
    fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
        input.bool()
    }
}

impl Encode for char {
    fn __encode(&self, out: &mut Encoder) {
        out.string(self.encode_utf8(&mut [0; 4]));
    }
}
impl Decode for char {
    fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
        input.ws();
        let start = input.pos;
        let s = input.string()?;
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(input.error_at(start, "invalid value: expected a string of one character")),
        }
    }
}

impl Encode for str {
    fn __encode(&self, out: &mut Encoder) {
        out.string(self);
    }
}
impl Encode for String {
    fn __encode(&self, out: &mut Encoder) {
        out.string(self);
    }
}
impl Decode for String {
    fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
        input.string()
    }
}

impl Decoder<'_> {
    /// A string checked by `parse` (`std::net::Url`'s decoding); an error points at its start.
    #[allow(dead_code)] // tide-admit includes this file without the types that use it.
    pub(crate) fn parsed<T, E: std::fmt::Display>(
        &mut self,
        parse: impl FnOnce(&str) -> Result<T, E>,
    ) -> Result<T, Error> {
        self.ws();
        let start = self.pos;
        let s = self.string()?;
        parse(&s).map_err(|e| self.error_at(start, format!("invalid value: {e}")))
    }
}

impl<T: Encode + ?Sized> Encode for &T {
    fn __encode(&self, out: &mut Encoder) {
        (**self).__encode(out);
    }
}

impl Encode for () {
    fn __encode(&self, out: &mut Encoder) {
        out.null();
    }
}
impl Decode for () {
    fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
        input.null()
    }
}

/// `None` is `null`.
impl<T: Encode> Encode for Option<T> {
    fn __encode(&self, out: &mut Encoder) {
        match self {
            Some(value) => value.__encode(out),
            None => out.null(),
        }
    }
}
/// `null` and an absent struct field are both `None`.
impl<T: Decode> Decode for Option<T> {
    fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
        input.ws();
        if input.peek() == Some(b'n') {
            input.literal(b"null").map(|()| None)
        } else {
            input.decode().map(Some)
        }
    }
    fn __missing(_: &Decoder<'_>, _: &str) -> Result<Self, Error> {
        Ok(None)
    }
}

impl<T: Encode + ?Sized> Encode for Box<T> {
    fn __encode(&self, out: &mut Encoder) {
        (**self).__encode(out);
    }
}
impl<T: Decode> Decode for Box<T> {
    fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
        input.decode().map(Box::new)
    }
    fn __missing(input: &Decoder<'_>, field: &str) -> Result<Self, Error> {
        T::__missing(input, field).map(Box::new)
    }
}

impl<T: Encode> Encode for [T] {
    fn __encode(&self, out: &mut Encoder) {
        out.array_begin();
        for value in self {
            out.element(value);
        }
        out.array_end();
    }
}
impl<T: Encode> Encode for Vec<T> {
    fn __encode(&self, out: &mut Encoder) {
        self.as_slice().__encode(out);
    }
}
impl<T: Encode> Encode for VecDeque<T> {
    fn __encode(&self, out: &mut Encoder) {
        out.array_begin();
        for value in self {
            out.element(value);
        }
        out.array_end();
    }
}

// Elements are pushed one at a time: no capacity is reserved from the input.
macro_rules! sequences {
    ($($t:ident $push:ident)*) => {$(
        impl<T: Decode> Decode for $t<T> {
            fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
                let mut out = $t::new();
                input.array_begin()?;
                while input.next_element()? {
                    out.$push(input.decode()?);
                }
                Ok(out)
            }
        }
    )*};
}
sequences!(Vec push VecDeque push_back);

macro_rules! tuples {
    ($(($len:literal $($t:ident $i:tt)+))*) => {$(
        impl<$($t: Encode),+> Encode for ($($t,)+) {
            fn __encode(&self, out: &mut Encoder) {
                out.array_begin();
                $(out.element(&self.$i);)+
                out.array_end();
            }
        }
        impl<$($t: Decode),+> Decode for ($($t,)+) {
            fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
                input.array_begin()?;
                let value = ($({
                    input.tuple_next($len)?;
                    input.decode::<$t>()?
                },)+);
                input.tuple_end($len)?;
                Ok(value)
            }
        }
    )*};
}
tuples! {
    (1 A 0)
    (2 A 0 B 1)
    (3 A 0 B 1 C 2)
    (4 A 0 B 1 C 2 D 3)
}

fn encode_map<'a, V: Encode + 'a>(
    out: &mut Encoder,
    entries: impl Iterator<Item = (&'a String, &'a V)>,
) {
    out.object_begin();
    for (key, value) in entries {
        out.field(key, value);
    }
    out.object_end();
}

/// Entries are written in key order (byte order), so equal maps encode to equal bytes
/// whatever their hasher, capacity, and insertion order.
impl<V: Encode, S> Encode for HashMap<String, V, S> {
    fn __encode(&self, out: &mut Encoder) {
        let mut entries: Vec<_> = self.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        encode_map(out, entries.into_iter());
    }
}
impl<V: Encode> Encode for BTreeMap<String, V> {
    fn __encode(&self, out: &mut Encoder) {
        encode_map(out, self.iter());
    }
}

// A repeated key is an error, where serde_json keeps the last value.
macro_rules! maps {
    ($(($($generics:tt)*) $t:ty;)*) => {$(
        impl<$($generics)*> Decode for $t {
            fn __decode(input: &mut Decoder<'_>) -> Result<Self, Error> {
                let mut out = Self::default();
                input.object_begin()?;
                while let Some(key) = input.next_key()? {
                    if out.contains_key(&key) {
                        return Err(input.error(format!("duplicate key `{key}`")));
                    }
                    let value = input.decode()?;
                    out.insert(key, value);
                }
                Ok(out)
            }
        }
    )*};
}
maps! {
    (V: Decode, S: BuildHasher + Default) HashMap<String, V, S>;
    (V: Decode) BTreeMap<String, V>;
}
