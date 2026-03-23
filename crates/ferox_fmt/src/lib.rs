#![forbid(unsafe_code)]

use ferox_syntax::SourceFile;

pub fn bootstrap_format(file: &SourceFile) -> String {
    let mut formatted = file.text.trim().to_string();
    if !formatted.ends_with('\n') {
        formatted.push('\n');
    }
    formatted
}

#[cfg(test)]
mod tests {
    use super::bootstrap_format;
    use ferox_syntax::{SourceFile, SourceFileId};

    #[test]
    fn bootstrap_format_trims_outer_whitespace_and_adds_newline() {
        let file = SourceFile::new(SourceFileId(1), "main.ferox", "  fn main() -> Int { 1 }  ");

        let formatted = bootstrap_format(&file);

        assert_eq!(formatted, "fn main() -> Int { 1 }\n");
    }
}

