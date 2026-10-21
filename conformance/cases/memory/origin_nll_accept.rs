//@ rules: ORIGIN-6
// NLL accepts this form of get-or-insert: no borrow is live across the insert.
use std::collections::HashMap;

pub fn get_or_insert(map: &mut HashMap<u32, String>) -> &String {
    if map.contains_key(&22) {
        return &map[&22];
    }
    map.insert(22, String::new());
    &map[&22]
}
