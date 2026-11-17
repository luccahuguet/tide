//@ rules: ADMIT-2
// Red team (fx-qlek.5): rustc rejects a `#![crate_name]` that differs from the driver's
// `--crate-name`, so a crate cannot rename itself (`escape_crate_type_accept` covers
// `#![crate_type]`).
#![crate_name = "renamed"] //~ error

pub fn noop() {}
