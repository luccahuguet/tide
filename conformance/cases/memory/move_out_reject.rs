//@ rules: MOVE-3
// Nothing moves out of a place behind a reference, or out of a field of a type with a
// destructor.
pub struct Node {
    pub name: String,
}

pub fn steal(n: &Node) -> String {
    n.name //~ E0507
}

pub fn deref(r: &String) -> String {
    *r //~ E0507
}

pub struct Guard {
    pub name: String,
}
impl Drop for Guard {
    fn drop(&mut self) {}
}

pub fn out_of_drop(g: Guard) -> String {
    g.name //~ E0509
}
