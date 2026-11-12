//@ rules: REF-0702
// Propagation, handling, return, and explicit discard consume a Result.
pub fn consume(result: Result<u8, ()>) -> Result<u8, ()> {
    let value = result?;
    let _ = Ok::<(), core::convert::Infallible>(());
    Ok(value)
}
pub fn ok(result: Result<u8, ()>) { result.ok(); }
pub fn handle(result: Result<u8, ()>) -> u8 { result.unwrap_or(0) }
pub fn write(mut text: String) -> std::fmt::Result {
    use std::fmt::Write;
    writeln!(text, "entry")?;
    let _ = writeln!(text, "discarded");
    Ok(())
}
#[must_use]
pub struct Notice;
pub fn other_must_use() { Notice; }
mod local { pub struct Result; }
pub fn same_spelling() { local::Result; }
