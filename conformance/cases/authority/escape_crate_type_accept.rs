//@ rules: ADMIT-2
// Red team (fx-qlek.5): `#![crate_type]` is admitted and inert: rustc ignores it when the driver
// passes `--crate-type`, so the build still makes what the driver asked for.
#![crate_type = "cdylib"]

pub fn noop() {}
