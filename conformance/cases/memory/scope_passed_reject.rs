//@ rules: SCOPE-4
// A child spawned on a passed-down scope cannot borrow the function's locals
// or a reference the signature does not tie to the scope.
use std::task::Scope;

pub struct Config {
    pub port: u16,
}

pub fn local<'s, E>(bg: Scope<'s, '_, E>) {
    let conn = String::from("x");
    bg.spawn(|| Ok(conn.len())); //~ E0373
}

pub fn untied<'s, E>(bg: Scope<'s, '_, E>, cfg: &Config) {
    bg.spawn(move || Ok(cfg.port)); //~ E0621
}
