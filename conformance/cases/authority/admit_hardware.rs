//@ rules: ADMIT-1
// Hardware queries are not admitted (03 ADMIT-1): `__cpuid` is safe to call but reads the
// machine. The case is x86_64-only, since `core::arch`'s modules are per target.
pub fn vendor() -> u32 {
    core::arch::x86_64::__cpuid(0).ebx //~ TD0520
}
