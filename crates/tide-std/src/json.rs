//! JSON for types that derive `Encode` and `Decode` (reference 2.2, bead `fx-08m`):
//! `encode`, `decode`, and `Error`. The conformance corpus's fixture std shares this file
//! through `root.rs`.
//!
//! The mapping is serde's. A struct with named fields is an object, in declaration order; a
//! tuple struct is an array; a one-field tuple struct is its field; a unit struct is `null`.
//! Enums are externally tagged: a unit variant is the string `"Clear"`, and a data variant a
//! single-key object, `{"Set":{"key":"a"}}`. `#[codec(..)]` takes `rename = ".."` (field or
//! variant), `rename_all = ".."` (struct or enum, with serde's eight case names), `default`
//! (named field or a struct represented as an object or array), and `skip` (named field).
//!
//! Decoding:
//!
//! Decoding is stricter than serde_json: a named struct written as an array and a unit
//! variant written as `{"Z":null}` are rejected.
//!
//! - A missing `Option` field is `None`, as is `null`, including through `Box` layers. Any
//!   other missing field is an error unless it or its struct has `default`; `null` is not an
//!   absent field. An explicit field `default` takes precedence over the struct default,
//!   including on a skipped field.
//!   A tuple struct with multiple fields uses its default to fill missing trailing array elements.
//! - Unknown keys are ignored, so they do not survive a round trip. A `skip` field's key is
//!   unknown; the field takes its default as above.
//! - A repeated struct field or map key is an error.
//! - Integers take no fraction or exponent and must fit their type. Floats take any JSON
//!   number and are rounded once; one too large for the type is an error, so a decoded float
//!   is finite.
//! - Strings are UTF-8 without raw control characters; `\u` surrogate pairs combine and a
//!   lone surrogate is an error. A byte-order mark is not accepted.
//! - Arrays and objects nest at most 128 deep, including inside ignored values.
//!   Independently, decoding allows at most 256 active codec calls, including transparent
//!   wrappers and derived newtypes, which need not consume any input; each `Option` or `Box`
//!   layer counts toward this budget.
//!
//! Encoding:
//!
//! - Output is compact UTF-8 without a trailing newline. Only `"`, `\`, and U+0000 to U+001F
//!   are escaped.
//! - Integers are written exactly at every width. Peers that read numbers as doubles lose
//!   integers beyond 2^53 (RFC 7493).
//! - Finite floats are written with the shortest digits that parse back to the same value,
//!   `-0.0` included. NaN and the infinities are written as `null`, which decodes into
//!   `Option<f64>` but not `f64`.
//! - `HashMap` entries are written in key order, so equal values give equal bytes. Map keys
//!   are `String`.
//!   A decoded `HashMap` uses the fixed default hash key, so decode input from untrusted peers
//!   into `BTreeMap`.
//! - `encode` recurses through the value, so a very deep value can exhaust the stack.

use crate::__codec::{Decode, Decoder, Encode, Encoder};
use std::fmt;

/// Why `decode` rejected its input: malformed JSON, or JSON that does not fit the type.
/// `Display` gives `<message> at line L column C`.
#[derive(Debug)]
pub struct Error {
    message: String,
    line: usize,
    column: usize,
}

impl Error {
    pub(crate) fn new(message: String, line: usize, column: usize) -> Self {
        Self {
            message,
            line,
            column,
        }
    }

    /// The line of the byte the error is at, from 1.
    pub fn line(&self) -> usize {
        self.line
    }

    /// The position of that byte in its line, counted in bytes from 1. At the end of the
    /// input it is one past the last byte.
    pub fn column(&self) -> usize {
        self.column
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {} column {}",
            self.message, self.line, self.column
        )
    }
}

impl std::error::Error for Error {}

/// `value` as compact JSON. It cannot fail.
pub fn encode<T: Encode + ?Sized>(value: &T) -> Vec<u8> {
    let mut out = Encoder::new();
    value.__encode(&mut out);
    out.finish()
}

/// The `T` that `bytes` holds: one JSON value of any kind, with only whitespace around it.
pub fn decode<T: Decode>(bytes: &[u8]) -> Result<T, Error> {
    let mut input = Decoder::new(bytes);
    let value = input.decode()?;
    input.finish()?;
    Ok(value)
}
