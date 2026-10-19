#![forbid(unsafe_code)]
fn main() {
    let r = core::arch::x86_64::__cpuid(1); // FX0520
    let _ = r.ebx >> 24; // FX0520 (the CpuidResult type)
}
