//@ rules: ORIGIN-6
// Polonius accepts this get-or-insert, but NLL keeps `v`'s loan live to the end of the
// function on every path, so v0 rejects it as rustc does.
use std::collections::HashMap;

pub fn get_or_insert(map: &mut HashMap<u32, String>) -> &String {
    if let Some(v) = map.get(&22) {
        return v;
    }
    map.insert(22, String::new()); //~ E0502
    &map[&22]
}
