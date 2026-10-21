//@ rules: PAR-4
// `par_map` takes `Pure + Sync` elements and `par_for` `Pure + Send` ones;
// owned strings are both.
pub fn lengths(words: &[String]) -> Vec<usize> {
    words.par_map(|w| w.len())
}

pub fn shout(words: &mut [String]) {
    words.par_for(|w| w.push('!'));
}
