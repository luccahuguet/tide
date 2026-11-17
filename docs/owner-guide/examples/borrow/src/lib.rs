// Owner's guide, chapter 2: a memory-safety error. Tide does not check this itself:
// rustc's borrow checker runs on every Tide build and rejects it (E0502).

pub fn grow_while_reading(names: &mut Vec<String>) -> usize {
    let first = &names[0]; // a shared borrow of the vector's contents...
    names.push(String::from("new")); // ...while it is changed: may move the contents
    first.len()
}
