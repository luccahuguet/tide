//@ rules: LIFE-8
// An associated type in an impl names the impl's lifetime.
pub struct Words<'a> {
    src: &'a str,
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let w = self.src;
        self.src = "";
        if w.is_empty() { None } else { Some(w) }
    }
}
