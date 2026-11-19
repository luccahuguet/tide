//@ rules: SCOPE-1, SCOPE-4, SEND-1, SEND-3
// Shared production/conformance lifecycle surface; the driver also builds and runs this binary.
use std::cell::Cell;
use std::task::{Scope, Task, TaskError, Tasks};

fn both<T: Send + Sync>() {}

fn traits<'s, 'env: 's, E>() {
    both::<Scope<'s, 'env, E>>();
    both::<Task<'s, Cell<u8>, Cell<u8>>>();
}

fn passed<'s, E>(s: Scope<'s, '_, E>, text: &'s str) {
    drop(s.spawn(move || Ok(text.len())));
}

fn chained<'s, E: Send + 's>(s: Scope<'s, '_, E>) {
    let child = s.spawn(|| Ok(4));
    drop(s.spawn(move || child.join()));
}

fn lifecycle(tasks: &Tasks) -> Result<(), TaskError> {
    traits::<std::rc::Rc<()>>();
    let mut count = 0;
    let text = String::from("lifecycle");
    let borrowed = tasks.scope::<_, TaskError, _>(|s| {
        passed(s, text.as_str());
        chained(s);
        drop(s.spawn(|| {
            count += 1;
            Ok(())
        }));
        std::mem::forget(s.spawn(move || {
            drop(s.spawn(|| {
                let local = std::rc::Rc::new(3);
                Ok(*local)
            }));
            Ok(())
        }));
        s.spawn(|| Ok(text.as_str())).join()
    })?;
    assert_eq!(borrowed, "lifecycle");
    assert_eq!(count, 1);
    Ok(())
}

#[tide::main]
fn main(root: Root) -> Result<(), TaskError> {
    lifecycle(&root.tasks)?;
    println!("TASKS_RAN");
    Ok(())
}
