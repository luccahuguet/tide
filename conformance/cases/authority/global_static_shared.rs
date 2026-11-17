//@ rules: GLOBAL-1
// A static must be `Pure`; a `Shared` handle is `Sync` but not `Pure`.
use std::sync::Shared;

pub static CELL: Option<Shared<u32>> = None; //~ TD0502
