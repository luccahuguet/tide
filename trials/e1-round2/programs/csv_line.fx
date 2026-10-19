//! Split one CSV line into fields, with RFC 4180 quoting.

use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CsvError {
    #[error("quoted field opened at byte {0} is never closed")]
    UnclosedQuote(usize),
    #[error("expected ',' or end of line after the closing quote, at byte {0}")]
    TextAfterQuote(usize),
    #[error("'\"' inside an unquoted field, at byte {0}")]
    QuoteInUnquoted(usize),
}

/// Splits `line` (without its line terminator) at commas. A field that
/// starts with `"` is quoted: it may contain commas, and `""` inside it
/// stands for one `"`. Fields borrow from `line`; only a field holding an
/// escaped quote is copied.
#[pure]
pub fn split_csv(line: &str) -> Result<Vec<Cow<str>>, CsvError>
    ensures match &result {
        Ok(fields) => !fields.is_empty(),
        Err(_) => true,
    }
{
    let bytes = line.as_bytes();
    let mut fields = Vec::new();
    let mut i = 0;                                      // start of the current field
    loop {
        if bytes.get(i) == Some(&b'"') {
            let (field, end) = quoted(line, i)?;
            fields.push(field);
            i = end;
            if i < bytes.len() && bytes[i] != b',' {
                return Err(CsvError::TextAfterQuote(i));
            }
        } else {
            let end = line[i..].find(',').map_or(line.len(), |k| i + k);
            if let Some(k) = line[i..end].find('"') {
                return Err(CsvError::QuoteInUnquoted(i + k));
            }
            fields.push(Cow::Borrowed(&line[i..end]));
            i = end;
        }
        if i == bytes.len() {
            return Ok(fields);
        }
        i += 1;                                         // skip the comma
    }
}

/// Parses the quoted field whose opening quote is at byte `open`. Returns
/// the field and the index just past its closing quote.
fn quoted(line: &str, open: usize) -> Result<(Cow<str>, usize), CsvError> {
    let mut owned: Option<String> = None;               // allocated at the first `""`
    let mut run = open + 1;                             // start of text not yet copied
    loop {
        let Some(k) = line[run..].find('"') else {
            return Err(CsvError::UnclosedQuote(open));
        };
        let q = run + k;
        if line.as_bytes().get(q + 1) == Some(&b'"') {
            owned.get_or_insert_with(String::new).push_str(&line[run..=q]);   // keeps one quote
            run = q + 2;
        } else {
            let field = match owned {
                None => Cow::Borrowed(&line[run..q]),
                Some(mut s) => {
                    s.push_str(&line[run..q]);
                    Cow::Owned(s)
                }
            };
            return Ok((field, q + 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OK_CASES: [(&str, &[&str]); 10] = [
        ("", &[""]),
        ("a", &["a"]),
        ("a,b,c", &["a", "b", "c"]),
        (",", &["", ""]),
        ("a,,c,", &["a", "", "c", ""]),
        (" a , b ", &[" a ", " b "]),
        ("\"x,y\",z", &["x,y", "z"]),
        ("\"say \"\"hi\"\"\",2", &["say \"hi\"", "2"]),
        ("\"\"", &[""]),
        ("\"\"\"\"", &["\""]),
    ];

    const BAD_CASES: [(&str, CsvError); 4] = [
        ("\"abc", CsvError::UnclosedQuote(0)),
        ("a,\"b\"\"", CsvError::UnclosedQuote(2)),
        ("\"a\"b", CsvError::TextAfterQuote(3)),
        ("ab\"c", CsvError::QuoteInUnquoted(2)),
    ];

    #[test]
    fn splits_valid_lines() {
        for (line, want) in OK_CASES {
            assert_eq!(split_csv(line).unwrap(), want, "line {line:?}");
        }
    }

    #[test]
    fn rejects_malformed_lines() {
        for (line, want) in BAD_CASES {
            assert_eq!(split_csv(line), Err(want), "line {line:?}");
        }
    }

    #[test]
    fn borrows_unless_unescaping() {
        let fields = split_csv("plain,\"quoted\",\"a\"\"b\"").unwrap();
        assert!(matches!(fields[0], Cow::Borrowed("plain")));
        assert!(matches!(fields[1], Cow::Borrowed("quoted")));
        assert!(matches!(fields[2], Cow::Owned(ref s) if s == "a\"b"));
    }

    /// Quotes a field when it needs it, as a CSV writer would.
    fn encode(fields: &[String]) -> String {
        let cells: Vec<String> = fields.iter()
            .map(|f| if f.contains([',', '"']) { format!("\"{}\"", f.replace('"', "\"\"")) } else { f.clone() })
            .collect();
        cells.join(",")
    }

    #[test]
    fn round_trips_through_a_writer(fields: Vec<String>)
        requires !fields.is_empty()
    {
        assert_eq!(split_csv(&encode(&fields)).unwrap(), fields);
    }
}
