//@ rules: SCOPE-4
// A child spawned on a passed-down scope owns its data or borrows what the
// signature ties to the scope.
use std::task::Scope;

pub struct Config {
    pub port: u16,
}

pub fn serve<'s, E>(bg: Scope<'s, '_, E>, cfg: &'s Config, conn: String) {
    bg.spawn(move || Ok(conn.len() + cfg.port as usize));
}
