// src/csv.fx of a library package
//
// Splits one CSV line (RFC 4180 quoting, no record terminator) into fields.
// Fields borrow from the line unless they contain an escaped quote (`""`),
// in which case the unescaped text is owned.

use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CsvError {
    #[error("quoted field starting at byte {start} is never closed")]
    UnterminatedQuote { start: usize },
    #[error("unexpected byte after a closing quote at byte {at}")]
    TrailingAfterQuote { at: usize },
    #[error("quote inside an unquoted field at byte {at}")]
    StrayQuote { at: usize },
}

/// Authority-free. Every returned field borrows from `line` (elision) or is owned.
#[pure]
pub fn split_csv_line(line: &str) -> Result<Vec<Cow<str>>, CsvError> {
    let bytes = line.as_bytes();
    let mut fields = Vec::new();
    let mut i = 0;
    loop {
        if bytes.get(i) == Some(&b'"') {
            // Quoted field. Indices next to ASCII bytes are always char boundaries.
            let start = i;
            i += 1;
            let content_start = i;
            let mut seg_start = i;
            let mut unescaped: Option<String> = None;
            loop {
                match bytes.get(i) {
                    None => return Err(CsvError::UnterminatedQuote { start }),
                    Some(b'"') if bytes.get(i + 1) == Some(&b'"') => {
                        // `""` stands for one `"`: keep the text up to and including the first.
                        unescaped.get_or_insert_with(String::new).push_str(&line[seg_start..i + 1]);
                        i += 2;
                        seg_start = i;
                    }
                    Some(b'"') => break,
                    Some(_) => i += 1,
                }
            }
            let field = match unescaped {
                None => Cow::Borrowed(&line[content_start..i]),
                Some(mut s) => {
                    s.push_str(&line[seg_start..i]);
                    Cow::Owned(s)
                }
            };
            fields.push(field);
            i += 1; // past the closing quote
            match bytes.get(i) {
                None => return Ok(fields),
                Some(b',') => i += 1,
                Some(_) => return Err(CsvError::TrailingAfterQuote { at: i }),
            }
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b',' {
                if bytes[i] == b'"' {
                    return Err(CsvError::StrayQuote { at: i });
                }
                i += 1;
            }
            fields.push(Cow::Borrowed(&line[start..i]));
            if i == bytes.len() {
                return Ok(fields);
            }
            i += 1; // past the comma
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_fields() {
        const CASES: [(&str, &[&str]); 10] = [
            ("", &[""]),
            ("a", &["a"]),
            ("a,b,c", &["a", "b", "c"]),
            ("a,,c", &["a", "", "c"]),
            ("a,", &["a", ""]),
            (" a , b ", &[" a ", " b "]),
            ("\"x,y\",z", &["x,y", "z"]),
            ("\"say \"\"hi\"\"\"", &["say \"hi\""]),
            ("\"\"", &[""]),
            ("\"\"\"\"", &["\""]),
        ];
        for (line, want) in CASES {
            let fields = split_csv_line(line).unwrap();
            let got: Vec<&str> = fields.iter().map(|f| f.as_ref()).collect();
            assert_eq!(got, want, "line {line:?}");
        }
    }

    #[test]
    fn rejects_malformed_lines() {
        const CASES: [(&str, CsvError); 4] = [
            ("\"abc", CsvError::UnterminatedQuote { start: 0 }),
            ("x,\"y", CsvError::UnterminatedQuote { start: 2 }),
            ("\"a\"b", CsvError::TrailingAfterQuote { at: 3 }),
            ("ab\"c", CsvError::StrayQuote { at: 2 }),
        ];
        for (line, want) in CASES {
            assert_eq!(split_csv_line(line), Err(want), "line {line:?}");
        }
    }

    #[test]
    fn borrows_unless_unescaping() {
        let fields = split_csv_line("plain,\"quoted\",\"a\"\"b\"").unwrap();
        assert!(matches!(fields[0], Cow::Borrowed("plain")));
        assert!(matches!(fields[1], Cow::Borrowed("quoted")));
        assert!(matches!(fields[2], Cow::Owned(ref s) if s == "a\"b"));
    }

    fn encode(fields: &[String]) -> String {
        fields
            .iter()
            .map(|f| format!("\"{}\"", f.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn quoted_round_trip(fields: Vec<String>) {
        if fields.is_empty() {
            return; // "" is one empty field, not zero fields
        }
        let line = encode(&fields);
        let got: Vec<String> = split_csv_line(&line).unwrap().into_iter().map(Cow::into_owned).collect();
        assert_eq!(got, fields);
    }
}
