use std::{collections::HashMap, fmt};
#[derive(Debug, PartialEq)]
pub enum TemplateError {
    Unclosed { pos: usize },
    BadName { pos: usize },
    StrayBrace { pos: usize },
    Missing { name: String },
}
impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unclosed { pos } => write!(f, "unclosed placeholder at {pos}"),
            Self::BadName { pos } => write!(f, "bad placeholder name at {pos}"),
            Self::StrayBrace { pos } => write!(f, "stray '}}' at {pos}"),
            Self::Missing { name } => write!(f, "missing value for {name}"),
        }
    }
}
enum Part<'a> {
    Text(&'a str),
    Name(&'a str),
}
pub struct Template<'a> {
    parts: Vec<Part<'a>>,
}
impl<'a> Template<'a> {
    pub fn parse(src: &'a str) -> Result<Self, TemplateError> {
        let mut parts = Vec::new();
        let mut pos = 0;
        while pos < src.len() {
            let s = &src[pos..];
            if s.starts_with("{{") || s.starts_with("}}") {
                parts.push(Part::Text(&src[pos..pos + 1]));
                pos += 2;
            } else if s.starts_with('{') {
                let end = s.find('}').ok_or(TemplateError::Unclosed { pos })?;
                let name = &s[1..end];
                if name.is_empty()
                    || !name.bytes().enumerate().all(|(i, b)| {
                        b == b'_' || b.is_ascii_alphabetic() || (i > 0 && b.is_ascii_digit())
                    })
                {
                    return Err(TemplateError::BadName { pos });
                }
                parts.push(Part::Name(name));
                pos += end + 1;
            } else if s.starts_with('}') {
                return Err(TemplateError::StrayBrace { pos });
            } else {
                let end = s.find(['{', '}']).unwrap_or(s.len());
                parts.push(Part::Text(&s[..end]));
                pos += end;
            }
        }
        Ok(Self { parts })
    }
    pub fn names(&self) -> Vec<&'a str> {
        let mut out = Vec::new();
        for p in &self.parts {
            if let Part::Name(n) = p {
                if !out.contains(n) {
                    out.push(*n);
                }
            }
        }
        out
    }
    pub fn render(&self, vars: &HashMap<&str, &str>) -> Result<String, TemplateError> {
        let mut out = String::new();
        for p in &self.parts {
            out.push_str(match p {
                Part::Text(s) => s,
                Part::Name(n) => vars.get(n).ok_or_else(|| TemplateError::Missing {
                    name: n.to_string(),
                })?,
            });
        }
        Ok(out)
    }
}
