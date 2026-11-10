//@ rules: SINK-8
// A library may use stderr and logging without the stdout lint.
pub fn report(n: u32) {
    eprint!("items: ");
    eprintln!("{n}");
    info!("items: {n}");
}
