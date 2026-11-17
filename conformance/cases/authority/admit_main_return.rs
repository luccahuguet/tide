//@ rules: ADMIT-1, GLOBAL-4, ROOT-1
// rustc's entry point reports the error `main` returns through its `Termination` impl, which
// formats it with `Debug`: here a function pointer's address.
fn noop() {}

fn main() -> Result<(), fn()> { //~ TD0512 TD0520
    Err(noop)
}
