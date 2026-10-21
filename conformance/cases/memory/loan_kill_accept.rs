//@ rules: LOAN-3
// Assigning to `list` kills the loans of places reached through `*list`, so each
// pass's `&mut list.value` does not conflict with the next one.
pub struct List {
    pub value: u32,
    pub next: Option<Box<List>>,
}

pub fn to_refs(mut list: &mut List) -> Vec<&mut u32> {
    let mut result = Vec::new();
    loop {
        result.push(&mut list.value);
        if let Some(n) = list.next.as_mut() {
            list = &mut **n;
        } else {
            return result;
        }
    }
}
