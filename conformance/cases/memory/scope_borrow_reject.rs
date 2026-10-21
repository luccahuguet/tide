//@ rules: SCOPE-1
// `F: σ` and `U: σ`: a child cannot borrow a local of the scope body, and the
// body cannot mutate, move, or borrow mutably a place a child borrows.
use std::task::{TaskError, Tasks};

pub fn body_local(tasks: &Tasks) -> Result<(), TaskError> {
    tasks.scope(|s| {
        let local = 5u64;
        s.spawn(|| Ok(local + 1)); //~ E0373
        Ok(())
    })
}

pub fn body_local_ref(tasks: &Tasks) -> Result<(), TaskError> {
    tasks.scope(|s| {
        let local = 5u64;
        let r = &local; //~ E0597
        s.spawn(move || Ok(*r + 1));
        Ok(())
    })
}

pub fn mutate(tasks: &Tasks) -> Result<(), TaskError> {
    let mut n = 0u64;
    tasks.scope(|s| {
        s.spawn(|| Ok(n + 1));
        n += 1; //~ E0506
        Ok(())
    })
}

// The move also makes `v` the body's own, so the child's borrow is E0373 too.
pub fn move_out(tasks: &Tasks) -> Result<(), TaskError> {
    let v = vec![1u64];
    tasks.scope(|s| {
        s.spawn(|| Ok(v.len())); //~ E0373
        drop(v); //~ E0505
        Ok(())
    })
}

pub fn second_mut(tasks: &Tasks) -> Result<(), TaskError> {
    let mut v = vec![1u64];
    tasks.scope(|s| {
        s.spawn(|| {
            v.push(2);
            Ok(())
        });
        v.push(3); //~ E0499
        Ok(())
    })
}
