//@ rules: LOAN-3, LOAN-2
// Assigning to `list` kills only the loans of places reached through `*list`:
// the loan of `other.value` survives it, so the next pass's borrow conflicts.
pub struct List {
    pub value: u32,
    pub next: Option<Box<List>>,
}

pub fn other_list<'a>(mut list: &'a mut List, other: &'a mut List) -> Vec<&'a mut u32> {
    let mut result = Vec::new();
    loop {
        result.push(&mut other.value); //~ E0499
        if let Some(n) = list.next.as_mut() {
            list = &mut **n;
        } else {
            return result;
        }
    }
}
