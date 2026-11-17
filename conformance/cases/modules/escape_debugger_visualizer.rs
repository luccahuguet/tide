//@ rules: REF-0911
// Red team (fx-qlek.5): `#![debugger_visualizer]` is stable and makes rustc read a file at
// compile time and embed it in the binary, like `include_bytes!`. It is not admitted.
#![debugger_visualizer(gdb_script_file = "escape_debugger_visualizer.rs")] //~ FX0911

pub fn noop() {}
