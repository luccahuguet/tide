//@ rules: ADMIT-1
// Red team (fx-qlek.5): `Termination::report`, which Tide's std re-exports for `main`'s return
// type, writes `Error: ..` to standard error. It is admitted: standard error is a sink every
// program holds (SINK-1), so this reaches no authority beyond `eprintln!`.
use std::process::Termination;

pub fn report() {
    let _ = Err::<(), &str>("written to stderr").report();
}
