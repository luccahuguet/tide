//@ rules: LIFE-8
// Associated types in impls have no elision.
pub struct Words<'a> {
    src: &'a str,
}

impl<'a> Iterator for Words<'a> {
    type Item = &str; //~ error
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
