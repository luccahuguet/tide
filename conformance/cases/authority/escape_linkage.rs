//@ rules: UNSAFE-1, GLOBAL-1
// Red team (fx-qlek.5): symbol interposition and static constructors. Exporting `malloc`
// replaces the allocator for the whole process, and a `.init_array` entry runs before `main`;
// both attributes need the `unsafe` grant, and a `static` holding a foreign function is not
// `Pure` either.
#[unsafe(export_name = "malloc")] //~ TD0901
pub extern "C" fn hijack() -> usize {
    0
}

extern "C" fn early() {}

#[used]
#[unsafe(link_section = ".init_array")]
static CONSTRUCTOR: extern "C" fn() = early; //~ TD0502 TD0901

// Interposition with `no_mangle` (Opus r2, M4) needs the grant too.
#[unsafe(no_mangle)] //~ TD0901
pub extern "C" fn free(_: usize) {}
