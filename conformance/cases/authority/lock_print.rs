//@ rules: LOCK-4, SINK-8
// Printing in each Shared callback is a warning; stdout also warns in a library.
use std::sync::{LockError, Shared};

pub fn report(a: &Shared<u32>, b: &Shared<u32>) -> Result<(), LockError> {
    a.with(|n| {
        println!("{n}"); //~ WARN FX0508 FX0607
        eprintln!("{n}"); //~ WARN FX0607
        let _ = dbg!(*n); //~ WARN FX0607
        (0..*n).for_each(|x| eprintln!("{x}")); //~ WARN FX0607
        let _later = || { eprintln!("not called"); }; //~ WARN FX0607
    })?;
    Shared::with2(a, b, |x, y| {
        eprint!("{x} {y}"); //~ WARN FX0607
    })?;
    let show = |xs: &mut [&mut u32]| {
        eprintln!("{}", xs.len()); //~ WARN FX0607
    };
    Shared::with_all(&[], &show)?;
    let boxed = Box::new(|n: &mut u32| {
        eprintln!("{n}"); //~ WARN FX0607
    });
    a.with(boxed)?;
    a.with(show_one)
}

fn show_one(n: &mut u32) {
    eprintln!("{n}"); //~ WARN FX0607
}
