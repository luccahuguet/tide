#![forbid(unsafe_code)]
#[doc = core::include_str!("/etc/hostname")] // FX0520 (expansion inside an attribute)
fn main() {
    let h = core::include_str!("/etc/hostname"); // FX0520
    let home = core::env!("HOME"); // FX0520
    let user = core::option_env!("USER"); // FX0520
    let _ = (h, home, user);
}
