//! `--tide-diff BASE HEAD`: what changed between two trust artifacts, a partial precursor to
//! `tide review`'s authority and trust-hole sections (03 §4.3, D11). It compares declared
//! authority and holes only: no exercised authority, effects, or proof status, and it says
//! nothing on whether a change is correct.
//!
//! An item is its path and kind, so a renamed or moved item is one removed and one added. An item
//! the head adds or removes is listed with its authority; one whose authority changed is
//! `widened`, `narrowed`, or both, with the kinds each path gained (`+`) or lost (`-`). A hole is
//! its item, kind, and what it is, and lists when it is new, gone, or its count changed. Two
//! artifacts with the same authority and holes give an empty report. The artifact's strings are
//! shown with control, line-separator, and bidirectional characters and `\` escaped, so none can
//! forge a line, and a path other than a plain one of names, positions, and `#<n>` is quoted, so
//! none reads as the value's `(value)`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::Path;

use super::{Authority, FORMAT};

/// An artifact as read: each item's authority, and each hole's count.
struct Artifact {
    items: BTreeMap<(String, String), Authority>,
    holes: BTreeMap<(String, String, String), u64>,
}

/// The report on what `head` changes from `base`, or why one cannot be read.
pub fn diff(base: &Path, head: &Path) -> Result<String, String> {
    Ok(compare(&read(base)?, &read(head)?))
}

// 03 §4.3: new paths to these kinds are listed first.
const FIRST: [&str; 3] = ["proc", "foreign", "accel"];
// 03 §4.3: a `proc` line says the child is not capability-checked.
const PROC_NOTE: &str = " (child is ambient; only names programs, 03 PROC-ONLY)";

fn compare(base: &Artifact, head: &Artifact) -> String {
    let mut blocks: Vec<(bool, String)> = Vec::new();
    let keys: BTreeSet<_> = base.items.keys().chain(head.items.keys()).collect();
    for key @ (item, kind) in keys {
        let (old, new) = (base.items.get(key), head.items.get(key));
        let label = match (old, new) {
            (None, Some(_)) => "+",
            (Some(_), None) => "-",
            (Some(old), Some(new)) if old != new => "~",
            _ => continue,
        };
        let empty = Authority::new();
        let (old, new) = (old.unwrap_or(&empty), new.unwrap_or(&empty));
        let mut lines = String::new();
        let (mut widened, mut narrowed, mut first) = (false, false, false);
        for path in old.keys().chain(new.keys()).collect::<BTreeSet<_>>() {
            let (was, is) = (old.get(path), new.get(path));
            for (sign, kinds, other, flag) in
                [("-", was, is, &mut narrowed), ("+", is, was, &mut widened)]
            {
                let kinds: Vec<_> = kinds
                    .into_iter()
                    .flatten()
                    .filter(|k| !other.is_some_and(|o| o.contains(*k)))
                    .map(|k| shown(k))
                    .collect();
                if !kinds.is_empty() {
                    *flag = true;
                    let added = sign == "+";
                    first |= added && kinds.iter().any(|k| FIRST.contains(&k.as_str()));
                    let note = if added && kinds.iter().any(|k| k == "proc") { PROC_NOTE } else { "" };
                    let path = shown_path(path);
                    let _ = writeln!(lines, "      {sign} {path}: {}{note}", kinds.join(", "));
                }
            }
        }
        let change = match (label, widened, narrowed) {
            ("~", true, true) => " widened and narrowed",
            ("~", true, false) => " widened",
            ("~", false, true) => " narrowed",
            _ => "",
        };
        let (item, kind) = (shown(item), shown(kind));
        let mut block = format!("  {label} {item} ({kind}){change}\n");
        block.push_str(&lines);
        blocks.push((first, block));
    }
    blocks.sort_by_key(|(first, _)| !first);
    let items: String = blocks.into_iter().map(|(_, block)| block).collect();
    let mut holes = String::new();
    let keys: BTreeSet<_> = base.holes.keys().chain(head.holes.keys()).collect();
    for key @ (item, kind, what) in keys {
        let (item, kind, what) = (shown(item), shown(kind), shown(what));
        let count = match (base.holes.get(key), head.holes.get(key)) {
            (None, Some(n)) => format!("+ {item} ({kind}): {what} ({n})"),
            (Some(n), None) => format!("- {item} ({kind}): {what} ({n})"),
            (Some(a), Some(b)) if a != b => format!("~ {item} ({kind}): {what} ({a} -> {b})"),
            _ => continue,
        };
        let _ = writeln!(holes, "  {count}");
    }
    let mut report = String::new();
    if !items.is_empty() {
        report.push_str("declared authority (no exercised authority, effects, or proof status)\n");
        report.push_str(&items);
    }
    if !holes.is_empty() {
        report.push_str("trust holes\n");
        report.push_str(&holes);
    }
    report
}

/// `s` with its control, line-separator, and bidirectional characters and `\` escaped.
fn shown(s: &str) -> String {
    s.chars()
        .map(|c| {
            let unseen = matches!(
                c,
                '\u{2028}'
                    | '\u{2029}'
                    | '\u{200e}'
                    | '\u{200f}'
                    | '\u{202a}'..='\u{202e}'
                    | '\u{2066}'..='\u{2069}'
            );
            if c.is_control() || unseen || c == '\\' {
                c.escape_default().to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}

/// An access path as the report shows it: `(value)` for the value itself, a plain path as it is,
/// and any other quoted.
fn shown_path(path: &str) -> String {
    if path.is_empty() {
        "(value)".into()
    } else if path
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '_' | '#' | '.'))
    {
        path.into()
    } else {
        format!("\"{}\"", shown(path).replace('"', "\\\""))
    }
}

/// Reads the artifact at `path`, which must be in this driver's `FORMAT`.
fn read(path: &Path) -> Result<Artifact, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn parse(text: &str) -> Result<Artifact, String> {
    let mut reader = Reader {
        text: text.as_bytes(),
        at: 0,
        depth: 0,
    };
    let json = reader.value()?;
    reader.space();
    if reader.at != reader.text.len() {
        return Err(reader.error("text after the artifact"));
    }
    let top = json.object("the artifact")?;
    let format = field(top, "format")?.string("`format`")?;
    if format != FORMAT {
        return Err(format!("format `{}`, not `{FORMAT}`", shown(format)));
    }
    let mut artifact = Artifact {
        items: BTreeMap::new(),
        holes: BTreeMap::new(),
    };
    for entry in field(top, "items")?.array("`items`")? {
        let entry = entry.object("an item")?;
        let key = (
            field(entry, "item")?.string("`item`")?.to_string(),
            field(entry, "kind")?.string("`kind`")?.to_string(),
        );
        let mut authority = Authority::new();
        for (path, kinds) in field(entry, "authority")?.object("`authority`")? {
            let kinds: BTreeSet<_> = kinds
                .array("a path's kinds")?
                .iter()
                .map(|k| k.string("a kind").map(str::to_string))
                .collect::<Result<_, _>>()?;
            if kinds.is_empty() {
                return Err(format!(
                    "path `{}` of `{}` lists no kinds",
                    shown(path),
                    shown(&key.0)
                ));
            }
            authority.insert(path.clone(), kinds);
        }
        if artifact.items.contains_key(&key) {
            let (item, kind) = (shown(&key.0), shown(&key.1));
            return Err(format!("item `{item}` ({kind}) is listed twice"));
        }
        artifact.items.insert(key, authority);
    }
    for entry in field(top, "holes")?.array("`holes`")? {
        let entry = entry.object("a hole")?;
        let key = (
            field(entry, "item")?.string("`item`")?.to_string(),
            field(entry, "kind")?.string("`kind`")?.to_string(),
            field(entry, "hole")?.string("`hole`")?.to_string(),
        );
        let count = field(entry, "count")?.number("`count`")?;
        if artifact.holes.insert(key.clone(), count).is_some() {
            let (what, item) = (shown(&key.2), shown(&key.0));
            return Err(format!("hole `{what}` of `{item}` is listed twice"));
        }
    }
    Ok(artifact)
}

fn field<'a>(object: &'a [(String, Json)], name: &str) -> Result<&'a Json, String> {
    object
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v)
        .ok_or_else(|| format!("missing `{name}`"))
}

/// The JSON the artifact uses: no booleans, nulls, fractions, or signs.
enum Json {
    String(String),
    Number(u64),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn string(&self, what: &str) -> Result<&str, String> {
        match self {
            Json::String(s) => Ok(s),
            _ => Err(format!("{what} is not a string")),
        }
    }
    fn number(&self, what: &str) -> Result<u64, String> {
        match self {
            Json::Number(n) => Ok(*n),
            _ => Err(format!("{what} is not a number")),
        }
    }
    fn array(&self, what: &str) -> Result<&[Json], String> {
        match self {
            Json::Array(a) => Ok(a),
            _ => Err(format!("{what} is not an array")),
        }
    }
    fn object(&self, what: &str) -> Result<&[(String, Json)], String> {
        match self {
            Json::Object(o) => Ok(o),
            _ => Err(format!("{what} is not an object")),
        }
    }
}

/// How deep arrays and objects may nest: the artifact needs five.
const MAX_NESTING: usize = 16;

struct Reader<'a> {
    text: &'a [u8],
    at: usize,
    depth: usize,
}

impl Reader<'_> {
    fn error(&self, what: &str) -> String {
        format!("malformed JSON at byte {}: {what}", self.at)
    }

    fn space(&mut self) {
        while self
            .text
            .get(self.at)
            .is_some_and(|b| b" \t\n\r".contains(b))
        {
            self.at += 1;
        }
    }

    /// Consumes `byte` after any whitespace, if it comes next.
    fn eat(&mut self, byte: u8) -> bool {
        self.space();
        let found = self.text.get(self.at) == Some(&byte);
        self.at += usize::from(found);
        found
    }

    fn value(&mut self) -> Result<Json, String> {
        self.space();
        match self.text.get(self.at) {
            Some(b'"') => self.string().map(Json::String),
            Some(b'0'..=b'9') => {
                let start = self.at;
                while self.text.get(self.at).is_some_and(u8::is_ascii_digit) {
                    self.at += 1;
                }
                let digits = &self.text[start..self.at];
                if digits.len() > 1 && digits[0] == b'0' {
                    return Err(self.error("a number with a leading zero"));
                }
                std::str::from_utf8(digits)
                    .ok()
                    .and_then(|d| d.parse().ok())
                    .map(Json::Number)
                    .ok_or_else(|| self.error("a number out of range"))
            }
            Some(b'[' | b'{') if self.depth == MAX_NESTING => Err(self.error("nesting too deep")),
            Some(b'[') => {
                self.at += 1;
                self.depth += 1;
                let mut values = Vec::new();
                if !self.eat(b']') {
                    loop {
                        values.push(self.value()?);
                        if self.eat(b']') {
                            break;
                        }
                        if !self.eat(b',') {
                            return Err(self.error("expected `,` or `]`"));
                        }
                    }
                }
                self.depth -= 1;
                Ok(Json::Array(values))
            }
            Some(b'{') => {
                self.at += 1;
                self.depth += 1;
                let mut fields = Vec::new();
                let mut keys = BTreeSet::new();
                if !self.eat(b'}') {
                    loop {
                        self.space();
                        if self.text.get(self.at) != Some(&b'"') {
                            return Err(self.error("expected a key"));
                        }
                        let key = self.string()?;
                        if !keys.insert(key.clone()) {
                            return Err(self.error("a key given twice"));
                        }
                        if !self.eat(b':') {
                            return Err(self.error("expected `:`"));
                        }
                        fields.push((key, self.value()?));
                        if self.eat(b'}') {
                            break;
                        }
                        if !self.eat(b',') {
                            return Err(self.error("expected `,` or `}`"));
                        }
                    }
                }
                self.depth -= 1;
                Ok(Json::Object(fields))
            }
            Some(_) => Err(self.error("unexpected value")),
            None => Err(self.error("unexpected end")),
        }
    }

    /// A string literal at the opening quote.
    fn string(&mut self) -> Result<String, String> {
        self.at += 1;
        let mut bytes = Vec::new();
        loop {
            let Some(&byte) = self.text.get(self.at) else {
                return Err(self.error("unterminated string"));
            };
            self.at += 1;
            match byte {
                b'"' => break,
                b'\\' => {
                    let escaped = match self.text.get(self.at) {
                        Some(b'"') => '"',
                        Some(b'\\') => '\\',
                        Some(b'/') => '/',
                        Some(b'n') => '\n',
                        Some(b't') => '\t',
                        Some(b'r') => '\r',
                        Some(b'b') => '\u{8}',
                        Some(b'f') => '\u{c}',
                        Some(b'u') => self.unicode()?,
                        _ => return Err(self.error("a bad escape")),
                    };
                    self.at += 1;
                    bytes.extend_from_slice(escaped.encode_utf8(&mut [0; 4]).as_bytes());
                }
                byte if byte < 0x20 => return Err(self.error("a control character in a string")),
                byte => bytes.push(byte),
            }
        }
        String::from_utf8(bytes).map_err(|_| self.error("a string that is not UTF-8"))
    }

    /// The character a `\u` escape at `u` gives, with a surrogate pair's second half; leaves the
    /// reader at the escape's last digit.
    fn unicode(&mut self) -> Result<char, String> {
        let high = self.hex(self.at + 1)?;
        self.at += 4;
        let code = match high {
            0xD800..=0xDBFF if self.text.get(self.at + 1..self.at + 3) == Some(b"\\u") => {
                let low = self.hex(self.at + 3)?;
                if !(0xDC00..=0xDFFF).contains(&low) {
                    return Err(self.error("an unpaired surrogate"));
                }
                self.at += 6;
                0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00)
            }
            code => code,
        };
        char::from_u32(code).ok_or_else(|| self.error("an unpaired surrogate"))
    }

    /// The four hex digits at `at`.
    fn hex(&self, at: usize) -> Result<u32, String> {
        self.text
            .get(at..at + 4)
            .filter(|h| h.iter().all(u8::is_ascii_hexdigit))
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u32::from_str_radix(h, 16).ok())
            .ok_or_else(|| self.error("a bad `\\u` escape"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(items: &str, holes: &str) -> String {
        format!(
            "{{\"format\": \"{FORMAT}\", \"crate\": \"lib\", \"items\": [{items}], \"holes\": [{holes}]}}"
        )
    }

    fn report(base: &str, head: &str) -> Result<String, String> {
        Ok(compare(&parse(base)?, &parse(head)?))
    }

    #[test]
    fn new_proc_paths_come_first_and_name_the_child_as_ambient() {
        let base = artifact(r##"{"item": "lib::a", "kind": "function", "authority": {}}"##, "");
        let head = artifact(
            r##"{"item": "lib::a", "kind": "function", "authority": {"#0": ["net"]}},
               {"item": "lib::run", "kind": "function", "authority": {"#0": ["proc"]}}"##,
            "",
        );
        let expected = "\
declared authority (no exercised authority, effects, or proof status)
  + lib::run (function)
      + #0: proc (child is ambient; only names programs, 03 PROC-ONLY)
  ~ lib::a (function) widened
      + #0: net
";
        assert_eq!(report(&base, &head).unwrap(), expected);
    }

    #[test]
    fn reports_widened_narrowed_added_removed_items_and_hole_changes() {
        let base = artifact(
            r##"{"item": "lib::f", "kind": "function", "authority": {"app.clock": ["clock"], "": ["fs"]}},
               {"item": "lib::gone", "kind": "function", "authority": {"#0": ["net"]}},
               {"item": "lib::same", "kind": "struct", "authority": {}}"##,
            r##"{"item": "lib::f", "kind": "function", "hole": "an `unsafe` block", "count": 1},
               {"item": "lib::g", "kind": "function", "hole": "uses `x`", "count": 2}"##,
        );
        let head = artifact(
            r##"{"item": "lib::f", "kind": "function", "authority": {"app.db.net": ["net"], "": ["fs", "net"]}},
               {"item": "lib::new", "kind": "function", "authority": {"#0": ["net", "sync"]}},
               {"item": "lib::same", "kind": "struct", "authority": {}}"##,
            r##"{"item": "lib::g", "kind": "function", "hole": "uses `x`", "count": 3},
               {"item": "lib::h", "kind": "function", "hole": "an `unsafe fn`", "count": 1}"##,
        );
        let expected = "\
declared authority (no exercised authority, effects, or proof status)
  ~ lib::f (function) widened and narrowed
      + (value): net
      - app.clock: clock
      + app.db.net: net
  - lib::gone (function)
      - #0: net
  + lib::new (function)
      + #0: net, sync
trust holes
  - lib::f (function): an `unsafe` block (1)
  ~ lib::g (function): uses `x` (2 -> 3)
  + lib::h (function): an `unsafe fn` (1)
";
        assert_eq!(report(&base, &head).unwrap(), expected);
        assert_eq!(report(&head, &head).unwrap(), "");
    }

    #[test]
    fn rejects_what_is_not_an_artifact_of_this_format() {
        let item = r##"{"item": "lib::f", "kind": "function", "authority": {}}"##;
        let good = artifact(item, "");
        let deep = format!("{}{}", "[".repeat(10_000), "]".repeat(10_000));
        for (bad, why) in [
            (
                good.replace(FORMAT, "tide-interface/2"),
                "format `tide-interface/2`",
            ),
            (artifact(&format!("{item}, {item}"), ""), "listed twice"),
            (
                good.replace("\"holes\": []", "\"holes\": [1]"),
                "a hole is not an object",
            ),
            (good.replace("\"holes\"", "\"hole\""), "missing `holes`"),
            (
                good.replace("\"crate\"", "\"items\": [], \"x\""),
                "a key given twice",
            ),
            (format!("{good} {{}}"), "text after the artifact"),
            (good[..good.len() - 1].to_string(), "expected `,` or `}`"),
            (good.replace("{}", "{\"a\": [true]}"), "unexpected value"),
            (good.replace("{}", "{\"a\": []}"), "lists no kinds"),
            (good.replace("[]", &deep), "nesting too deep"),
            (good.replace("lib::f", "lib::\\q"), "a bad escape"),
            (good.replace("lib::f", "lib::\\u+041"), "a bad `\\u` escape"),
            (
                good.replace("lib::f", "lib::\\ud801"),
                "an unpaired surrogate",
            ),
            (
                artifact(
                    "",
                    r#"{"item": "f", "kind": "k", "hole": "h", "count": 01}"#,
                ),
                "a leading zero",
            ),
        ] {
            let err = report(&good, &bad).unwrap_err();
            assert!(err.contains(why), "{why}: {err}");
        }
        let escaped = good.replace("lib::f", "lib::\\ud801\\udc00\\u0066");
        assert_eq!(
            report(&good.replace("lib::f", "lib::\u{10400}f"), &escaped).unwrap(),
            ""
        );
    }

    /// No string in an artifact can forge a report line or read as another path.
    #[test]
    fn shows_strings_that_cannot_forge_report_lines() {
        let forged = artifact(
            r#"{"item": "lib::f\n  - lib::g (function)\u001b[2J\u2028\u202e", "kind": "function", "authority": {"(value)": ["net"], "a\nb": ["fs"]}}"#,
            "",
        );
        let expected = "\
declared authority (no exercised authority, effects, or proof status)
  + lib::f\\n  - lib::g (function)\\u{1b}[2J\\u{2028}\\u{202e} (function)
      + \"(value)\": net
      + \"a\\nb\": fs
";
        assert_eq!(report(&artifact("", ""), &forged).unwrap(), expected);
    }
}
